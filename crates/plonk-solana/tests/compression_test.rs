#![cfg(feature = "vk")]

mod common;

use plonk_solana::{verify, PlonkError};

#[test]
fn test_compression_roundtrip() {
    let proof = common::load_test_proof();
    let compressed = proof.compress().unwrap();
    let decompressed = compressed.decompress().unwrap();
    assert_eq!(
        proof, decompressed,
        "Proof mismatch after compress/decompress roundtrip"
    );
}

#[test]
fn test_compressed_proof_verifies() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let public_inputs = common::load_test_public_inputs();

    let compressed = proof.compress().unwrap();
    let decompressed = compressed.decompress().unwrap();
    verify(&vk, &decompressed, &public_inputs).unwrap();
}

#[test]
fn test_decompress_fails_invalid_point() {
    let proof = common::load_test_proof();
    let mut compressed = proof.compress().unwrap();
    // Corrupt the 'a' point with a value exceeding the field modulus
    compressed.a = [0xFF; 32];
    let result = compressed.decompress();
    assert_eq!(
        result,
        Err(PlonkError::G1DecompressionFailed),
        "Expected G1DecompressionFailed for invalid compressed point"
    );
}

// Note: G1CompressionFailed is not testable -- the underlying alt_bn128_g1_compress_be
// syscall does not validate curve membership, so it never returns an error for
// valid-length (64-byte) inputs.
