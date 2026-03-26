use light_program_profiler::profile;
use plonk_solana::plonk::{Challenges, Proof, VerificationKey};
use plonk_solana::{Fr, PlonkError};

#[profile]
pub fn bench_transcript_get_challenge() -> Result<Fr, PlonkError> {
    use plonk_solana::g1::G1;
    let scalar_bytes = Fr::one().to_be_bytes();
    plonk_solana::transcript::hash_challenge(&[&G1::GENERATOR.0, &scalar_bytes])
}

#[profile]
pub fn bench_calculate_challenges(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; 1],
) -> Result<Challenges, PlonkError> {
    plonk_solana::plonk::calculate_challenges(vk, proof, public_inputs)
}
