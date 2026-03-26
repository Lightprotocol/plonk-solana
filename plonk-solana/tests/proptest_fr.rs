use plonk_solana::{bigint_from_be_bytes, bigint_to_be_bytes, Fr};
use proptest::prelude::*;

proptest! {
    #[test]
    fn fr_canonical_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        if let Some(fr) = Fr::from_be_bytes(&bytes) {
            prop_assert_eq!(fr.to_be_bytes(), bytes);
        }
    }

    #[test]
    fn fr_unchecked_idempotent(bytes in prop::array::uniform32(any::<u8>())) {
        let fr = Fr::from_be_bytes_unchecked(&bytes);
        let canonical = fr.to_be_bytes();
        let fr2 = Fr::from_be_bytes_unchecked(&canonical);
        prop_assert_eq!(fr, fr2);
    }

    #[test]
    fn bigint_bytes_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        let bigint = bigint_from_be_bytes(&bytes);
        prop_assert_eq!(bigint_to_be_bytes(&bigint), bytes);
    }
}
