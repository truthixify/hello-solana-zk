use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::Groth16Verifier;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
use std::ops::Neg;
use verifying_key::VERIFYINGKEY;
use proof::{PUBLIC_SIGNALS, PROOF};

mod proof;
mod verifying_key;

type G1 = ark_bn254::g1::G1Affine;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let proof_a: G1 = G1::deserialize_with_mode(
        &*[&change_endianness(&PROOF[0..64]), &[0u8][..]].concat(),
        Compress::No,
        Validate::Yes,
    )
    .map_err(|_| ProgramError::InvalidInstructionData)?;
    let mut proof_a_neg = [0u8; 65];
    proof_a
        .neg()
        .x
        .serialize_with_mode(&mut proof_a_neg[..32], Compress::No)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    proof_a
        .neg()
        .y
        .serialize_with_mode(&mut proof_a_neg[32..], Compress::No)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let proof_a: [u8; 64] = change_endianness(&proof_a_neg[..64]).try_into().unwrap();
    let proof_b = &_instruction_data[64..192]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let proof_c = &_instruction_data[192..256]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let public_signals: [[u8; 32]; 1] = chunk_instruction_data(&_instruction_data[256..])
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let mut verifier =
        Groth16Verifier::new(&proof_a, proof_b, proof_c, &public_signals, &VERIFYINGKEY)
            .map_err(|_| ProgramError::Custom(0))?;

    match verifier.verify() {
        Ok(true) => {
            msg!("Proof verification succeeded");
        }
        Ok(false) => {
            msg!("Proof is invalid");
        }
        Err(err) => {
            msg!("Proof verification failed: {:?}", err);
            return Err(ProgramError::InvalidInstructionData);
        }
    }
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
    use solana_program_test::*;
    use solana_sdk::{signature::Signer,
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_transaction() {
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) =
            ProgramTest::new("program", program_id, processor!(process_instruction))
                .start()
                .await;

        // Create instruction data using valid proof and public inputs
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&PROOF);
        for input in PUBLIC_SIGNALS.iter() {
            instruction_data.extend_from_slice(input); // Valid public inputs
        }

        // Create the instruction to invoke the program
        let instruction = solana_program::instruction::Instruction::new_with_bytes(
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
