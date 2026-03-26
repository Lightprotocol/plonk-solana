#![cfg(feature = "bench")]

use plonk_solana::plonk::{g1_add, g1_mul, g1_neg};
use plonk_solana::{Fr, G1};
use proptest::prelude::*;

proptest! {
    #[test]
    fn g1_neg_additive_inverse(scalar_bytes in prop::array::uniform32(any::<u8>())) {
        let scalar = Fr::from_be_bytes_unchecked(&scalar_bytes);
        if scalar == Fr::zero() { return Ok(()); }
        let p = g1_mul(&G1::GENERATOR, &scalar).unwrap();
        let neg_p = g1_neg(&p);
        let sum = g1_add(&p, &neg_p).unwrap();
        prop_assert_eq!(sum, G1::ZERO);
    }

    #[test]
    fn g1_neg_double_negation(scalar_bytes in prop::array::uniform32(any::<u8>())) {
        let scalar = Fr::from_be_bytes_unchecked(&scalar_bytes);
        if scalar == Fr::zero() { return Ok(()); }
        let p = g1_mul(&G1::GENERATOR, &scalar).unwrap();
        prop_assert_eq!(p, g1_neg(&g1_neg(&p)));
    }
}
