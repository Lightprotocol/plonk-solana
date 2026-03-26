use light_program_profiler::profile;
use plonk_solana::g1::G1;
use plonk_solana::plonk::{self, Challenges, Proof, VerificationKey};
use plonk_solana::{Fr, PlonkError};

#[profile]
pub fn bench_calculate_l1_and_pi(
    vk: &VerificationKey,
    ch: &Challenges,
    public_inputs: &[Fr; 1],
) -> Result<(Fr, Fr), PlonkError> {
    plonk::calculate_l1_and_pi(vk, ch, public_inputs)
}

#[profile]
pub fn bench_calculate_r0_and_d(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    l1: &Fr,
    pi: &Fr,
    eval_bytes: &[[u8; 32]; 6],
) -> Result<(Fr, G1), PlonkError> {
    plonk::calculate_r0_and_d(vk, proof, ch, l1, pi, eval_bytes)
}

#[profile]
pub fn bench_calculate_f(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    d: &G1,
) -> Result<G1, PlonkError> {
    plonk::calculate_f(vk, proof, ch, d)
}

#[profile]
pub fn bench_is_valid_pairing(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    r0: &Fr,
    f: &G1,
) -> Result<bool, PlonkError> {
    plonk::is_valid_pairing(vk, proof, ch, r0, f)
}
