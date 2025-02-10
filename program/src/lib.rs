use ark_bn254::{self, Bn254};
use ark_ff::BigInteger;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::Groth16Verifier;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
use verifying_key::get_vkey_from_json;
use std::{fs::{read, File}, ops::Neg};
use ark_groth16::VerifyingKey;

mod verifying_key;

type G1 = ark_bn254::g1::G1Affine;
type G2 = ark_bn254::g2::G2Affine;

entrypoint!(process_instruction);

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    for b in bytes.chunks(32) {
        for byte in b.iter().rev() {
            vec.push(*byte);
        }
    }
    vec
}

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // msg!("vkey: {:?}", VERIFYING_KEY);
    let vkey: VerifyingKey<Bn254> = get_vkey_from_json();
    msg!("vkey: {:?}", vkey);
    let _instruction_data = &_instruction_data[4..];

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
    use solana_program_test::*;
    use solana_sdk::{signature::Signer, transaction::Transaction};

    const PROOF: [u8; 256] = [
        220, 67, 64, 26, 46, 89, 237, 206, 15, 62, 21, 122, 24, 223, 42, 22, 243, 14, 211, 63, 155,
        236, 28, 25, 32, 228, 117, 2, 230, 187, 197, 28, 179, 146, 150, 73, 225, 36, 169, 139, 248,
        70, 127, 213, 77, 188, 167, 221, 120, 181, 83, 95, 208, 45, 205, 14, 155, 176, 35, 35, 32,
        228, 41, 17, 35, 12, 113, 210, 84, 88, 150, 59, 160, 143, 203, 121, 207, 25, 220, 96, 134,
        155, 19, 132, 162, 132, 221, 233, 233, 88, 24, 2, 92, 192, 47, 38, 250, 162, 185, 124, 165,
        29, 200, 187, 206, 167, 52, 42, 6, 240, 209, 168, 129, 247, 90, 169, 187, 129, 200, 9, 16,
        143, 113, 222, 12, 64, 93, 35, 77, 46, 70, 170, 56, 155, 35, 25, 87, 239, 245, 249, 9, 201,
        86, 100, 92, 209, 252, 123, 140, 49, 227, 218, 120, 237, 11, 222, 53, 222, 188, 34, 40,
        233, 251, 97, 197, 229, 23, 206, 38, 253, 106, 16, 64, 170, 215, 127, 112, 250, 155, 109,
        139, 38, 226, 241, 117, 229, 195, 194, 119, 104, 178, 31, 254, 161, 121, 197, 188, 225,
        199, 97, 128, 203, 57, 6, 74, 252, 222, 0, 41, 75, 233, 228, 192, 16, 91, 12, 219, 113, 0,
        154, 141, 117, 194, 2, 223, 245, 198, 78, 158, 129, 38, 138, 83, 169, 99, 130, 74, 119,
        230, 159, 21, 197, 180, 231, 25, 125, 185, 11, 189, 30, 119, 60, 231, 212, 234, 38,
    ];

    pub const PUBLIC_INPUTS: [[u8; 32]; 1] = [[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 12,
    ]];

    // Helper function to create a mock AccountInfo with a specified lifetime
    fn mock_account_info<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo::new(key, false, true, lamports, data, owner, false, 0)
    }

    #[tokio::test]
    async fn test_transaction() {
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) =
            ProgramTest::new("program", program_id, processor!(process_instruction))
                .start()
                .await;

        // Create instruction data using valid proof and public inputs
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&PROOF); // Valid proof
        for input in PUBLIC_INPUTS.iter() {
            instruction_data.extend_from_slice(input); // Valid public inputs
        }

        // Create the instruction to invoke the program
        let instruction = solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &instruction_data,
            vec![],
        );

        // Add the instruction to a new transaction
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // Process the transaction
        let transaction_result = banks_client.process_transaction(transaction).await;
        assert!(transaction_result.is_ok());
    }
}
