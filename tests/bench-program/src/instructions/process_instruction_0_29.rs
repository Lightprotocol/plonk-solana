use pinocchio::ProgramResult;

use crate::benchmarks::{fr_ops, g1_ops, transcript_ops};
use crate::instructions::discriminator::PlonkBenchInstruction;
use crate::{test_proof, test_public_inputs_bytes, test_public_inputs_fr, verifying_key};

#[inline(never)]
pub fn process_instruction_0_29(instruction: PlonkBenchInstruction) -> ProgramResult {
    match instruction {
        PlonkBenchInstruction::Baseline => {
            crate::baseline_empty_function();
        }
        PlonkBenchInstruction::G1Add => {
            let proof = test_proof();
            let _ = g1_ops::bench_g1_add(&proof.a, &proof.b).unwrap();
        }
        PlonkBenchInstruction::G1Neg => {
            let proof = test_proof();
            let _ = g1_ops::bench_g1_neg(&proof.a);
        }
        PlonkBenchInstruction::G1Mul => {
            let proof = test_proof();
            let _ = g1_ops::bench_g1_mul(&proof.a, &proof.eval_a).unwrap();
        }
        PlonkBenchInstruction::G1Compress => {
            let proof = test_proof();
            let _ = g1_ops::bench_g1_compress(&proof.a).unwrap();
        }
        PlonkBenchInstruction::G1Decompress => {
            let proof = test_proof();
            let compressed = proof.a.compress().unwrap();
            let _ = g1_ops::bench_g1_decompress(&compressed).unwrap();
        }
        PlonkBenchInstruction::FrFromBeBytes => {
            let bytes = test_public_inputs_bytes();
            let _ = fr_ops::bench_fr_from_be_bytes(&bytes[0]);
        }
        PlonkBenchInstruction::FrToBeBytes => {
            let inputs = test_public_inputs_fr();
            let _ = fr_ops::bench_fr_to_be_bytes(&inputs[0]);
        }
        PlonkBenchInstruction::FrSquare => {
            let inputs = test_public_inputs_fr();
            let _ = fr_ops::bench_fr_square(&inputs[0]);
        }
        PlonkBenchInstruction::FrInverse => {
            let inputs = test_public_inputs_fr();
            let _ = fr_ops::bench_fr_inverse(&inputs[0]);
        }
        PlonkBenchInstruction::FrAdd => {
            let proof = test_proof();
            let _ = fr_ops::bench_fr_add(proof.eval_a, proof.eval_b);
        }
        PlonkBenchInstruction::FrSub => {
            let proof = test_proof();
            let _ = fr_ops::bench_fr_sub(proof.eval_a, proof.eval_b);
        }
        PlonkBenchInstruction::FrMul => {
            let proof = test_proof();
            let _ = fr_ops::bench_fr_mul(proof.eval_a, proof.eval_b);
        }
        PlonkBenchInstruction::IsLessThanFieldSize => {
            let bytes = test_public_inputs_bytes();
            let _ = fr_ops::bench_is_less_than_field_size(&bytes[0]);
        }
        PlonkBenchInstruction::TranscriptGetChallenge => {
            let _ = transcript_ops::bench_transcript_get_challenge().unwrap();
        }
        PlonkBenchInstruction::CalculateChallenges => {
            let vk = verifying_key();
            let proof = test_proof();
            let inputs = test_public_inputs_fr();
            let eval_bytes = plonk_solana::plonk::compute_eval_bytes(&proof);
            let _ = transcript_ops::bench_calculate_challenges(&vk, &proof, &inputs, &eval_bytes)
                .unwrap();
        }
        _ => return Err(pinocchio::error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
