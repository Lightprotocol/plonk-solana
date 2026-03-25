use light_program_profiler::profile;
use plonk_solana::Fr;

#[profile]
pub fn bench_fr_from_be_bytes(bytes: &[u8; 32]) -> Option<Fr> {
    Fr::from_be_bytes(bytes)
}

#[profile]
pub fn bench_fr_to_be_bytes(fr: &Fr) -> [u8; 32] {
    fr.to_be_bytes()
}

#[profile]
pub fn bench_fr_square(fr: &Fr) -> Fr {
    fr.square()
}

#[profile]
pub fn bench_fr_inverse(fr: &Fr) -> Option<Fr> {
    fr.inverse()
}

#[profile]
pub fn bench_fr_add(a: Fr, b: Fr) -> Fr {
    a + b
}

#[profile]
pub fn bench_fr_sub(a: Fr, b: Fr) -> Fr {
    a - b
}

#[profile]
pub fn bench_fr_mul(a: Fr, b: Fr) -> Fr {
    a * b
}

#[profile]
pub fn bench_is_less_than_field_size(bytes: &[u8; 32]) -> bool {
    plonk_solana::is_less_than_bn254_field_size_be(bytes)
}
