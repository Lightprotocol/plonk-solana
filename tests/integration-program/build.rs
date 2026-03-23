fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    plonk_solana::vk_parser::generate_vk_file(
        "test-fixtures/verification_key.json",
        &out_dir,
        "verifying_key.rs",
    )
    .unwrap();
}
