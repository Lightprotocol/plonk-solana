use light_program_profiler::profile;
use plonk_solana::plonk::{Challenges, Proof, VerificationKey};
use plonk_solana::{Fr, PlonkError};

#[profile]
pub fn bench_transcript_get_challenge() -> Result<Fr, PlonkError> {
    use plonk_solana::g1::G1;
    use plonk_solana::transcript::Transcript;
    let mut transcript = Transcript::new();
    transcript.add_point(&G1::GENERATOR);
    transcript.add_scalar(&Fr::one());
    transcript.get_challenge()
}

#[profile]
pub fn bench_calculate_challenges(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; 1],
) -> Result<Challenges, PlonkError> {
    plonk_solana::plonk::calculate_challenges(vk, proof, public_inputs)
}
