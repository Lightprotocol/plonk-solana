use crate::errors::PlonkError;
use solana_bn254::compression::prelude::{alt_bn128_g1_compress_be, alt_bn128_g1_decompress_be};

/// Uncompressed G1 point on BN254: 64 bytes big-endian (x || y).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck_derive::Pod, bytemuck_derive::Zeroable))]
#[cfg_attr(feature = "zerocopy", derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout, zerocopy::Unaligned))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[repr(transparent)]
pub struct G1(pub [u8; 64]);

/// Compressed G1 point on BN254: 32 bytes big-endian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck_derive::Pod, bytemuck_derive::Zeroable))]
#[cfg_attr(feature = "zerocopy", derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout, zerocopy::Unaligned))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
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

#[cfg(feature = "serde")]
impl serde::Serialize for G1 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for G1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct G1Visitor;
        impl<'de> serde::de::Visitor<'de> for G1Visitor {
            type Value = G1;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("64 bytes")
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<G1, E> {
                let bytes: [u8; 64] = v.try_into().map_err(|_| {
                    E::invalid_length(v.len(), &"64 bytes")
                })?;
                Ok(G1(bytes))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<G1, A::Error> {
                let mut bytes = [0u8; 64];
                for (i, b) in bytes.iter_mut().enumerate() {
                    *b = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(i, &"64 bytes")
                    })?;
                }
                Ok(G1(bytes))
            }
        }
        deserializer.deserialize_bytes(G1Visitor)
    }
}
