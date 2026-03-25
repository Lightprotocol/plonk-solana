fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    plonk_solana::vk_parser::generate_vk_file(
        "../fixtures/data/verification_key.json",
        &out_dir,
        "verifying_key.rs",
    )
    .unwrap();
    plonk_solana::vk_parser::generate_proof_file(
        "../fixtures/data/proof.json",
        &out_dir,
        "test_proof.rs",
    )
    .unwrap();
    plonk_solana::vk_parser::generate_public_inputs_file(
        "../fixtures/data/public.json",
        &out_dir,
        "test_public_inputs.rs",
    )
    .unwrap();
}
