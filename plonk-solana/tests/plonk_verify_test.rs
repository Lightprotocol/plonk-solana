#![cfg(feature = "vk")]

mod common;

use plonk_solana::{verify, PlonkError};

#[test]
fn test_verify_valid_proof() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let public_input = common::load_test_public_input_bytes();
    verify(&vk, &proof, &[public_input]).unwrap();
}

#[test]
fn test_verify_fails_invalid_public_input() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let bad_input = {
        let mut b = [0u8; 32];
        b[31] = 34;
        b
    };
    assert_eq!(
        verify(&vk, &proof, &[bad_input]),
        Err(PlonkError::ProofVerificationFailed),
        "Expected ProofVerificationFailed for wrong public input value"
    );
}

#[test]
fn test_verify_fails_too_many_public_inputs() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    // VK has n_public=1, pass 2 inputs
    assert_eq!(
        verify(&vk, &proof, &[[0u8; 32], [0u8; 32]]),
        Err(PlonkError::InvalidPublicInputsLength),
        "Expected InvalidPublicInputsLength for too many inputs"
    );
}

#[test]
fn test_verify_fails_empty_public_inputs() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    assert_eq!(
        verify::<0>(&vk, &proof, &[]),
        Err(PlonkError::InvalidPublicInputsLength),
        "Expected InvalidPublicInputsLength for empty inputs"
    );
}
