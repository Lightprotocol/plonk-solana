#![cfg(feature = "vk")]

mod common;

use plonk_solana::{verify, Fr, PlonkError};

#[test]
fn test_verify_valid_proof() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let public_inputs = common::load_test_public_inputs();
    verify(&vk, &proof, &public_inputs).unwrap();
}

#[test]
fn test_verify_fails_invalid_public_input() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let bad_inputs = vec![Fr::from(34u64)];
    assert_eq!(
        verify(&vk, &proof, &bad_inputs),
        Err(PlonkError::ProofVerificationFailed),
        "Expected ProofVerificationFailed for wrong public input value"
    );
}

#[test]
fn test_verify_fails_too_many_public_inputs() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    // VK has n_public=1, pass 2 inputs
    let too_many = vec![Fr::from(15u64), Fr::from(42u64)];
    assert_eq!(
        verify(&vk, &proof, &too_many),
        Err(PlonkError::InvalidPublicInputsLength),
        "Expected InvalidPublicInputsLength for too many inputs"
    );
}

#[test]
fn test_verify_fails_empty_public_inputs() {
    let vk = common::load_test_vk();
    let proof = common::load_test_proof();
    let empty: Vec<Fr> = vec![];
    assert_eq!(
        verify(&vk, &proof, &empty),
        Err(PlonkError::InvalidPublicInputsLength),
        "Expected InvalidPublicInputsLength for empty inputs"
    );
}
