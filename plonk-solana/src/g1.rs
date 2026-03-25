use crate::errors::PlonkError;
use solana_bn254::compression::prelude::{alt_bn128_g1_compress_be, alt_bn128_g1_decompress_be};

/// Uncompressed G1 point on BN254: 64 bytes big-endian (x || y).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct G1(pub [u8; 64]);

/// Compressed G1 point on BN254: 32 bytes big-endian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompressedG1(pub [u8; 32]);

impl G1 {
    pub const ZERO: Self = Self([0u8; 64]);

    pub const GENERATOR: Self = {
        let mut g = [0u8; 64];
        g[31] = 1;
        g[63] = 2;
        Self(g)
    };

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn compress(&self) -> Result<CompressedG1, PlonkError> {
        let bytes =
            alt_bn128_g1_compress_be(&self.0).map_err(|_| PlonkError::G1CompressionFailed)?;
        Ok(CompressedG1(bytes))
    }
}

impl CompressedG1 {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn decompress(&self) -> Result<G1, PlonkError> {
        let bytes =
            alt_bn128_g1_decompress_be(&self.0).map_err(|_| PlonkError::G1DecompressionFailed)?;
        Ok(G1(bytes))
    }
}

impl AsRef<[u8; 64]> for G1 {
    fn as_ref(&self) -> &[u8; 64] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for CompressedG1 {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 64]> for G1 {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 32]> for CompressedG1 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&G1> for CompressedG1 {
    type Error = PlonkError;

    fn try_from(point: &G1) -> Result<Self, PlonkError> {
        point.compress()
    }
}

impl TryFrom<&CompressedG1> for G1 {
    type Error = PlonkError;

    fn try_from(point: &CompressedG1) -> Result<Self, PlonkError> {
        point.decompress()
    }
}
