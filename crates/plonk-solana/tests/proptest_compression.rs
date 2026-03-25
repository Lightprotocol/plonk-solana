use plonk_solana::{compress_g1, decompress_g1};
use proptest::prelude::*;
use solana_bn254::prelude::alt_bn128_g1_multiplication_be;

fn g1_from_scalar(scalar_bytes: [u8; 32]) -> [u8; 64] {
    let g1_gen = {
        let mut g = [0u8; 64];
        g[31] = 1;
        g[63] = 2;
        g
    };
    let mut input = [0u8; 96];
    input[..64].copy_from_slice(&g1_gen);
    input[64..].copy_from_slice(&scalar_bytes);
    alt_bn128_g1_multiplication_be(&input)
        .unwrap()
        .try_into()
        .unwrap()
}

proptest! {
    #[test]
    fn test_g1_compression_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        let point = g1_from_scalar(bytes);
        let compressed = compress_g1(&point).unwrap();
        let decompressed = decompress_g1(&compressed).unwrap();
        prop_assert_eq!(decompressed, point, "G1 roundtrip failed for scalar {:?}", bytes);
    }
}
