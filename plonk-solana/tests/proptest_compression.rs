use plonk_solana::G1;
use proptest::prelude::*;

fn g1_from_scalar(scalar_bytes: [u8; 32]) -> G1 {
    let mut input = [0u8; 96];
    input[..64].copy_from_slice(&G1::GENERATOR.0);
    input[64..].copy_from_slice(&scalar_bytes);
    let bytes = plonk_solana::syscalls::g1_multiplication_be(&input).unwrap();
    G1(bytes)
}

proptest! {
    #[test]
    fn test_g1_compression_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        let point = g1_from_scalar(bytes);
        let compressed = point.compress().unwrap();
        let decompressed = compressed.decompress().unwrap();
        prop_assert_eq!(decompressed, point, "G1 roundtrip failed for scalar {:?}", bytes);
    }
}
