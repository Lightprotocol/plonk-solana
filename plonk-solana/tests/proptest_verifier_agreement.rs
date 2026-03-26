#![cfg(all(feature = "vk", feature = "bench"))]

use plonk_solana::Fr;
use proptest::prelude::*;

mod common;

fn solana_fr_to_ark(fr: &Fr) -> ark_bn254::Fr {
    let bytes = fr.to_be_bytes();
    let mut le = bytes;
    le.reverse();
    ark_bn254::Fr::from(ark_ff::BigInt::<4>([
        u64::from_le_bytes(le[0..8].try_into().unwrap()),
        u64::from_le_bytes(le[8..16].try_into().unwrap()),
        u64::from_le_bytes(le[16..24].try_into().unwrap()),
        u64::from_le_bytes(le[24..32].try_into().unwrap()),
    ]))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn verifiers_agree_on_mutated_eval(
        field_idx in 0..6usize,
        random_bytes in prop::array::uniform32(any::<u8>()),
    ) {
        let s_vk = common::load_test_vk();
        let mut s_proof = common::load_test_proof();
        let s_input = common::load_test_public_inputs()[0];

        let random_fr = Fr::from_be_bytes_unchecked(&random_bytes);
        let ark_random_fr = solana_fr_to_ark(&random_fr);

        // Mutate one eval field in plonk-solana proof
        match field_idx {
            0 => s_proof.eval_a = random_fr,
            1 => s_proof.eval_b = random_fr,
            2 => s_proof.eval_c = random_fr,
            3 => s_proof.eval_s1 = random_fr,
            4 => s_proof.eval_s2 = random_fr,
            5 => s_proof.eval_zw = random_fr,
            _ => unreachable!(),
        }

        let solana_ok = plonk_solana::verify_unchecked(&s_vk, &s_proof, &[s_input]).is_ok();

        // Build arkworks proof with same mutation
        let vk_json: plonk_verifier::parse::VkJson = serde_json::from_str(
            include_str!("../../tests/fixtures/data/verification_key.json"),
        ).unwrap();
        let proof_json: plonk_verifier::parse::ProofJson = serde_json::from_str(
            include_str!("../../tests/fixtures/data/proof.json"),
        ).unwrap();
        let a_vk = vk_json.parse();
        let mut a_proof = proof_json.parse();
        let a_inputs = plonk_verifier::parse::parse_public_inputs(
            include_str!("../../tests/fixtures/data/public.json"),
        );

        match field_idx {
            0 => a_proof.eval_a = ark_random_fr,
            1 => a_proof.eval_b = ark_random_fr,
            2 => a_proof.eval_c = ark_random_fr,
            3 => a_proof.eval_s1 = ark_random_fr,
            4 => a_proof.eval_s2 = ark_random_fr,
            5 => a_proof.eval_zw = ark_random_fr,
            _ => unreachable!(),
        }

        let ark_ok = plonk_verifier::verify(&a_vk, &a_proof, &a_inputs);

        prop_assert_eq!(
            solana_ok, ark_ok,
            "verifiers disagree: solana={}, arkworks={}, field_idx={}, random_fr={:?}",
            solana_ok, ark_ok, field_idx, random_bytes
        );
    }
}
