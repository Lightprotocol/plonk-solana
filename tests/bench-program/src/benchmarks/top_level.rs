use light_program_profiler::profile;
use plonk_solana::plonk::{CompressedProof, Proof, VerificationKey};
use plonk_solana::{Fr, PlonkError};

#[profile]
pub fn bench_verify(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[[u8; 32]; 1],
) -> Result<(), PlonkError> {
    plonk_solana::verify(vk, proof, public_inputs)
}

#[profile]
pub fn bench_verify_unchecked(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; 1],
) -> Result<(), PlonkError> {
    plonk_solana::verify_unchecked(vk, proof, public_inputs)
}

#[profile]
pub fn bench_proof_compress(proof: &Proof) -> Result<CompressedProof, PlonkError> {
    proof.compress()
}

#[profile]
pub fn bench_proof_decompress(proof: &CompressedProof) -> Result<Proof, PlonkError> {
    proof.decompress()
}
