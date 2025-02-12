use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::{groth16::Groth16Verifier, errors::Groth16Error};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
use std::ops::Neg;
use verifier::verify_proof;

mod verifier;

type G1 = ark_bn254::g1::G1Affine;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let mut verifier_result = verify_proof(_instruction_data);

    match verifier_result {
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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
    use solana_program_test::*;
    use solana_sdk::{signature::Signer,
        transaction::Transaction,
    };
    
    pub const PROOF: [u8; 256] = [11,153,22,210,131,29,22,33,109,32,118,255,191,108,172,62,255,105,221,19,123,108,143,200,50,126,91,218,114,93,241,167,41,41,77,49,83,34,162,161,12,222,102,120,199,223,171,176,222,3,115,126,128,46,108,86,241,54,27,172,241,37,38,164,46,246,248,242,72,136,83,185,246,109,60,79,44,253,129,184,160,245,74,180,236,131,245,67,225,0,5,54,228,102,8,127,47,94,196,66,76,230,96,201,244,210,232,141,171,215,121,174,221,27,6,220,111,135,7,80,143,168,214,45,29,245,227,123,1,21,200,245,226,233,109,237,144,71,22,216,128,105,89,1,93,206,233,240,249,197,106,228,54,49,184,120,212,41,236,151,19,8,98,112,80,171,12,214,115,125,214,3,29,237,208,198,107,100,194,114,167,179,242,225,16,183,142,160,102,4,4,179,8,185,88,74,52,184,88,33,227,66,171,105,51,155,206,189,108,210,46,204,18,142,171,60,230,177,82,186,79,240,179,152,0,76,118,156,103,132,169,229,234,186,0,46,95,140,51,174,181,91,241,65,166,237,3,246,217,198,29,22,125,2,235,17];

    pub const PUBLIC_SIGNALS: [[u8; 32]; 1] = [[15,184,73,247,207,53,134,92,131,140,239,72,120,46,128,59,44,56,38,62,47,70,119,153,200,126,255,22,142,180,216,151]];

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
