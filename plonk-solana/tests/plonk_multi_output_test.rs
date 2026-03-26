#![cfg(feature = "vk")]

use plonk_solana::vk_parser;
use plonk_solana::Fr;

macro_rules! test_circuit {
    ($name:ident, $n:literal, $vk_path:expr, $proof_path:expr, $public_path:expr) => {
        mod $name {
            use super::*;

            fn load() -> (
                plonk_solana::VerificationKey,
                plonk_solana::Proof,
                [Fr; $n],
                [u8; 32],
            ) {
                let vk = vk_parser::parse_vk_json(include_str!($vk_path)).unwrap();
                let proof = vk_parser::parse_proof_json(include_str!($proof_path)).unwrap();
                let inputs =
                    vk_parser::parse_public_inputs_json(include_str!($public_path)).unwrap();
                assert_eq!(inputs.len(), $n, "expected {} public inputs", $n);
                let fr_arr: [Fr; $n] = core::array::from_fn(|i| inputs[i]);
                let first_bytes = inputs[0].to_be_bytes();
                (vk, proof, fr_arr, first_bytes)
            }

            #[test]
            fn verify_valid_proof() {
                let (vk, proof, inputs, _) = load();
                let byte_inputs: [[u8; 32]; $n] = core::array::from_fn(|i| inputs[i].to_be_bytes());
                plonk_solana::verify(&vk, &proof, &byte_inputs).unwrap();
            }

            #[test]
            fn verify_unchecked_valid_proof() {
                let (vk, proof, inputs, _) = load();
                plonk_solana::verify_unchecked(&vk, &proof, &inputs).unwrap();
            }

            #[test]
            fn verify_rejects_wrong_input() {
                let (vk, proof, mut inputs, _) = load();
                inputs[0] = Fr::from(999u64);
                let result = plonk_solana::verify_unchecked(&vk, &proof, &inputs);
                assert!(result.is_err(), "should reject wrong public input");
            }

            #[test]
            fn matches_arkworks_reference() {
                let vk_json = include_str!($vk_path);
                let proof_json = include_str!($proof_path);
                let public_json = include_str!($public_path);

                // plonk-solana
                let s_vk = vk_parser::parse_vk_json(vk_json).unwrap();
                let s_proof = vk_parser::parse_proof_json(proof_json).unwrap();
                let s_inputs_vec = vk_parser::parse_public_inputs_json(public_json).unwrap();
                let s_inputs: [Fr; $n] = core::array::from_fn(|i| s_inputs_vec[i]);
                let result = plonk_solana::verify_unchecked(&s_vk, &s_proof, &s_inputs);
                assert!(result.is_ok(), "plonk-solana rejected valid proof");

                // arkworks reference
                let a_vk: plonk_verifier::parse::VkJson = serde_json::from_str(vk_json).unwrap();
                let a_proof: plonk_verifier::parse::ProofJson =
                    serde_json::from_str(proof_json).unwrap();
                let a_inputs = plonk_verifier::parse::parse_public_inputs(public_json);
                assert!(
                    plonk_verifier::verify(&a_vk.parse(), &a_proof.parse(), &a_inputs),
                    "arkworks rejected valid proof"
                );
            }
        }
    };
}

test_circuit!(
    mul1,
    1,
    "../../tests/fixtures/data/mul1/verification_key.json",
    "../../tests/fixtures/data/mul1/proof.json",
    "../../tests/fixtures/data/mul1/public.json"
);

test_circuit!(
    mul2,
    2,
    "../../tests/fixtures/data/mul2/verification_key.json",
    "../../tests/fixtures/data/mul2/proof.json",
    "../../tests/fixtures/data/mul2/public.json"
);

test_circuit!(
    mul3,
    3,
    "../../tests/fixtures/data/mul3/verification_key.json",
    "../../tests/fixtures/data/mul3/proof.json",
    "../../tests/fixtures/data/mul3/public.json"
);

test_circuit!(
    mul4,
    4,
    "../../tests/fixtures/data/mul4/verification_key.json",
    "../../tests/fixtures/data/mul4/proof.json",
    "../../tests/fixtures/data/mul4/public.json"
);

test_circuit!(
    mul5,
    5,
    "../../tests/fixtures/data/mul5/verification_key.json",
    "../../tests/fixtures/data/mul5/proof.json",
    "../../tests/fixtures/data/mul5/public.json"
);
