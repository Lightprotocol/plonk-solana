use light_program_profiler::profile;
use plonk_solana::g1::{CompressedG1, G1};
use plonk_solana::plonk;
use plonk_solana::Fr;
use plonk_solana::PlonkError;

#[profile]
pub fn bench_g1_add(a: &G1, b: &G1) -> Result<G1, PlonkError> {
    plonk::g1_add(a, b)
}

#[profile]
pub fn bench_g1_neg(p: &G1) -> G1 {
    plonk::g1_neg(p)
}

#[profile]
pub fn bench_g1_mul(point: &G1, scalar: &Fr) -> Result<G1, PlonkError> {
    plonk::g1_mul(point, scalar)
}

#[profile]
pub fn bench_g1_compress(point: &G1) -> Result<CompressedG1, PlonkError> {
    point.compress()
}

#[profile]
pub fn bench_g1_decompress(point: &CompressedG1) -> Result<G1, PlonkError> {
    point.decompress()
}
