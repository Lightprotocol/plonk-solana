/// Thin wrapper around ark_bn254::Fr for convenient byte conversion.
/// All serialization uses 32-byte big-endian format (matching snarkjs/EIP-197).
use ark_bn254::Fr as ArkFr;
use ark_ff::{Field, One, PrimeField, Zero};
use num_bigint::BigUint;

/// Returns true if the big-endian byte representation is less than the BN254
/// scalar field (Fr) modulus. Use this to reject non-canonical public inputs
/// before converting to Fr.
pub fn is_less_than_bn254_field_size_be(bytes: &[u8; 32]) -> bool {
    let n = BigUint::from_bytes_be(bytes);
    n < <ArkFr as PrimeField>::MODULUS.into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fr(pub ArkFr);

impl Fr {
    pub fn zero() -> Self {
        Self(ArkFr::zero())
    }

    pub fn one() -> Self {
        Self(ArkFr::one())
    }

    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let n = BigUint::from_bytes_be(bytes);
        Self(ArkFr::from(n))
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let n: BigUint = self.0.into();
        let bytes = n.to_bytes_be();
        let mut result = [0u8; 32];
        let start = 32usize.saturating_sub(bytes.len());
        result[start..].copy_from_slice(&bytes);
        result
    }

    pub fn square(&self) -> Self {
        Self(self.0.square())
    }

    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(Self)
    }
}

impl std::ops::Add for Fr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Fr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul for Fr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Neg for Fr {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl From<u64> for Fr {
    fn from(v: u64) -> Self {
        Self(ArkFr::from(v))
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Fr {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Fr {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(Fr::from_be_bytes(&buf))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Fr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.to_be_bytes())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Fr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FrVisitor;
        impl<'de> serde::de::Visitor<'de> for FrVisitor {
            type Value = Fr;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("32 bytes")
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Fr, E> {
                let bytes: [u8; 32] = v.try_into().map_err(|_| {
                    E::invalid_length(v.len(), &"32 bytes")
                })?;
                Ok(Fr::from_be_bytes(&bytes))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Fr, A::Error> {
                let mut bytes = [0u8; 32];
                for (i, b) in bytes.iter_mut().enumerate() {
                    *b = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(i, &"32 bytes")
                    })?;
                }
                Ok(Fr::from_be_bytes(&bytes))
            }
        }
        deserializer.deserialize_bytes(FrVisitor)
    }
}
