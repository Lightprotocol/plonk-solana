use crate::errors::PlonkError;
use solana_bn254::compression::prelude::{alt_bn128_g1_compress_be, alt_bn128_g1_decompress_be};

/// Decompress a G1 point from 32 bytes to 64 bytes (big-endian).
pub fn decompress_g1(bytes: &[u8; 32]) -> Result<[u8; 64], PlonkError> {
    alt_bn128_g1_decompress_be(bytes).map_err(|_| PlonkError::G1DecompressionFailed)
}

/// Compress a G1 point from 64 bytes to 32 bytes (big-endian).
pub fn compress_g1(bytes: &[u8; 64]) -> Result<[u8; 32], PlonkError> {
    alt_bn128_g1_compress_be(bytes).map_err(|_| PlonkError::G1CompressionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// G1 generator: (1, 2) in big-endian.
    const G1_GENERATOR: [u8; 64] = {
        let mut g = [0u8; 64];
        g[31] = 1;
        g[63] = 2;
        g
    };

    #[test]
    fn test_g1_compress_decompress_roundtrip() {
        let compressed = compress_g1(&G1_GENERATOR).unwrap();
        assert_ne!(compressed, [0u8; 32]);
        let decompressed = decompress_g1(&compressed).unwrap();
        assert_eq!(decompressed, G1_GENERATOR);
    }

    #[test]
    fn test_g1_identity_compression() {
        let zero = [0u8; 64];
        let compressed = compress_g1(&zero).unwrap();
        assert_eq!(compressed, [0u8; 32]);
        let decompressed = decompress_g1(&compressed).unwrap();
        assert_eq!(decompressed, zero);
    }
}
