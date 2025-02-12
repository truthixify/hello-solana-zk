use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::{groth16::{Groth16Verifier, Groth16Verifyingkey}, errors::Groth16Error};
use std::ops::Neg;
type G1 = ark_bn254::g1::G1Affine;

const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
	nr_pubinputs: 2,

	vk_alpha_g1: [
		45,77,154,167,227,2,217,223,65,116,157,85,7,148,157,5,219,234,51,251,177,108,100,59,34,245,153,162,190,109,242,226,
		20,190,221,80,60,55,206,176,97,216,236,96,32,159,227,69,206,137,131,10,25,35,3,1,240,118,202,255,0,77,25,38,
	],

	vk_beta_g2: [
		9,103,3,47,203,247,118,209,175,201,133,248,136,119,241,130,211,132,128,166,83,242,222,202,169,121,76,188,59,243,6,12,
		14,24,120,71,173,76,121,131,116,208,214,115,43,245,1,132,125,214,139,192,224,113,36,30,2,19,188,127,193,61,183,171,
		48,76,251,209,224,138,112,74,153,245,232,71,217,63,140,60,170,253,222,196,107,122,13,55,157,166,154,77,17,35,70,167,
		23,57,193,177,164,87,168,199,49,49,35,210,77,47,145,146,248,150,183,198,62,234,5,169,213,127,6,84,122,208,206,200,
	],

	vk_gamme_g2: [
		25,142,147,147,146,13,72,58,114,96,191,183,49,251,93,37,241,170,73,51,53,169,231,18,151,228,133,183,174,243,18,194,
		24,0,222,239,18,31,30,118,66,106,0,102,94,92,68,121,103,67,34,212,247,94,218,221,70,222,189,92,217,146,246,237,
		9,6,137,208,88,95,240,117,236,158,153,173,105,12,51,149,188,75,49,51,112,179,142,243,85,172,218,220,209,34,151,91,
		18,200,94,165,219,140,109,235,74,171,113,128,141,203,64,143,227,209,231,105,12,67,211,123,76,230,204,1,102,250,125,170,
	],

	vk_delta_g2: [
		25,197,198,201,41,255,205,210,158,139,16,92,126,117,244,77,113,179,82,6,200,151,139,141,34,86,184,183,27,245,50,78,
		0,23,152,198,111,90,191,220,170,175,162,143,246,238,51,74,87,101,46,132,239,49,62,109,63,121,175,205,103,77,208,116,
		10,203,57,164,220,106,188,45,41,131,75,63,172,208,7,208,42,236,238,174,9,177,194,123,110,138,85,189,43,20,38,247,
		31,203,55,38,124,132,56,223,216,120,16,16,171,181,52,72,76,1,206,46,94,61,112,151,106,46,47,63,81,246,227,188,
	],

	vk_ic: &[
		[
			29,56,102,180,146,100,53,146,125,237,136,108,0,182,108,62,220,100,34,22,103,108,110,70,62,52,140,213,216,98,225,192,
			27,238,159,229,253,32,134,124,18,114,212,95,82,214,28,17,93,118,255,103,73,6,81,222,76,6,59,109,159,187,128,134,
		],
		[
			46,97,14,189,7,112,34,79,98,104,141,196,58,136,157,127,151,122,78,98,230,254,49,98,26,147,124,185,61,172,165,94,
			12,142,209,38,187,128,226,231,162,126,105,194,211,228,75,235,81,233,199,186,27,89,6,42,81,97,136,126,218,57,147,225,
		],
	]
};

fn chunk_instruction_data(data: &[u8]) -> Vec<[u8; 32]> {
    data.chunks(32)
        .map(|chunk| {
            let mut array = [0u8; 32];
            array[..chunk.len()].copy_from_slice(chunk);
            array
        })
        .collect()
}

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    for b in bytes.chunks(32) {
        for byte in b.iter().rev() {
            vec.push(*byte);
        }
    }
    vec
}

pub fn verify_proof(data: &[u8]) -> Result<bool, Groth16Error> {
    let proof_a: G1 = G1::deserialize_with_mode(
        &*[&change_endianness(&data[0..64]), &[0u8][..]].concat(),
        Compress::No,
        Validate::Yes,
    )
    .map_err(|_| Groth16Error::DecompressingG1Failed)?;
    let mut proof_a_neg = [0u8; 65];
    proof_a
        .neg()
        .x
        .serialize_with_mode(&mut proof_a_neg[..32], Compress::No)
        .map_err(|_| Groth16Error::DecompressingG1Failed)?;
    proof_a
        .neg()
        .y
        .serialize_with_mode(&mut proof_a_neg[32..], Compress::No)
        .map_err(|_| Groth16Error::DecompressingG1Failed)?;

    let proof_a: [u8; 64] = change_endianness(&proof_a_neg[..64]).try_into().map_err(|_| Groth16Error::InvalidG1Length)?;
    let proof_b = &data[64..192]
        .try_into().map_err(|_| Groth16Error::InvalidG2Length)?;
    let proof_c = &data[192..256]
        .try_into().map_err(|_| Groth16Error::InvalidG1Length)?;
    let public_signals: [[u8; 32]; 1] = chunk_instruction_data(&data[256..])
        .try_into().map_err(|_| Groth16Error::InvalidPublicInputsLength)?;

    let mut verifier =
        Groth16Verifier::new(&proof_a, proof_b, proof_c, &public_signals, &VERIFYINGKEY).map_err(|_| Groth16Error::ProofVerificationFailed)?;
        
    verifier.verify()
}
