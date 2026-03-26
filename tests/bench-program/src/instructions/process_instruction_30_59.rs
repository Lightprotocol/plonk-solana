use pinocchio::ProgramResult;

use crate::benchmarks::{top_level, verification_ops};
use crate::instructions::discriminator::PlonkBenchInstruction;
use crate::{test_proof, test_public_inputs_bytes, test_public_inputs_fr, verifying_key};

#[inline(never)]
pub fn process_instruction_30_59(instruction: PlonkBenchInstruction) -> ProgramResult {
    match instruction {
        PlonkBenchInstruction::CalculateL1AndPi => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            let eval_bytes = plonk_solana::plonk::compute_eval_bytes(&proof);
            let ch = plonk_solana::plonk::calculate_challenges(&vk, &proof, &inputs, &eval_bytes)
                .unwrap();
            let _ = verification_ops::bench_calculate_l1_and_pi(&vk, &ch, &inputs).unwrap();
        }
        PlonkBenchInstruction::CalculateR0AndD => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            let eval_bytes = plonk_solana::plonk::compute_eval_bytes(&proof);
            let ch = plonk_solana::plonk::calculate_challenges(&vk, &proof, &inputs, &eval_bytes)
                .unwrap();
            let (l1, pi) = plonk_solana::plonk::calculate_l1_and_pi(&vk, &ch, &inputs).unwrap();
            let _ =
                verification_ops::bench_calculate_r0_and_d(&vk, &proof, &ch, &l1, &pi, &eval_bytes)
                    .unwrap();
        }
        PlonkBenchInstruction::CalculateF => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            let eval_bytes = plonk_solana::plonk::compute_eval_bytes(&proof);
            let ch = plonk_solana::plonk::calculate_challenges(&vk, &proof, &inputs, &eval_bytes)
                .unwrap();
            let (l1, pi) = plonk_solana::plonk::calculate_l1_and_pi(&vk, &ch, &inputs).unwrap();
            let (_r0, d) =
                plonk_solana::plonk::calculate_r0_and_d(&vk, &proof, &ch, &l1, &pi, &eval_bytes)
                    .unwrap();
            let _ = verification_ops::bench_calculate_f(&vk, &proof, &ch, &d).unwrap();
        }
        PlonkBenchInstruction::IsValidPairing => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            let eval_bytes = plonk_solana::plonk::compute_eval_bytes(&proof);
            let ch = plonk_solana::plonk::calculate_challenges(&vk, &proof, &inputs, &eval_bytes)
                .unwrap();
            let (l1, pi) = plonk_solana::plonk::calculate_l1_and_pi(&vk, &ch, &inputs).unwrap();
            let (r0, d) =
                plonk_solana::plonk::calculate_r0_and_d(&vk, &proof, &ch, &l1, &pi, &eval_bytes)
                    .unwrap();
            let f = plonk_solana::plonk::calculate_f(&vk, &proof, &ch, &d).unwrap();
            let _ = verification_ops::bench_is_valid_pairing(&vk, &proof, &ch, &r0, &f).unwrap();
        }
        PlonkBenchInstruction::Verify => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_bytes();
            top_level::bench_verify(&vk, &proof, &inputs).unwrap();
        }
        PlonkBenchInstruction::VerifyUnchecked => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            top_level::bench_verify_unchecked(&vk, &proof, &inputs).unwrap();
        }
        PlonkBenchInstruction::ProofCompress => {
            let proof = test_proof();
            let _ = top_level::bench_proof_compress(&proof).unwrap();
        }
        PlonkBenchInstruction::ProofDecompress => {
            let proof = test_proof();
            let compressed = proof.compress().unwrap();
            let _ = top_level::bench_proof_decompress(&compressed).unwrap();
        }
        _ => return Err(pinocchio::error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
