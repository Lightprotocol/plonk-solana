/// Thin wrapper around ark_bn254::Fr for convenient byte conversion.
/// All serialization uses 32-byte big-endian format (matching snarkjs/EIP-197).
use ark_bn254::Fr as ArkFr;
use ark_ff::{BigInt, Field, One, PrimeField, Zero};

/// Convert 32 big-endian bytes to ark BigInt<4> (little-endian u64 limbs).
pub fn bigint_from_be_bytes(bytes: &[u8; 32]) -> BigInt<4> {
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        let offset = 24 - i * 8;
        *limb = u64::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
    }
    BigInt(limbs)
}

/// Convert ark BigInt<4> to 32 big-endian bytes.
pub fn bigint_to_be_bytes(n: &BigInt<4>) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..4 {
        let offset = 24 - i * 8;
        let limb_bytes = n.0[i].to_be_bytes();
        bytes[offset] = limb_bytes[0];
        bytes[offset + 1] = limb_bytes[1];
        bytes[offset + 2] = limb_bytes[2];
        bytes[offset + 3] = limb_bytes[3];
        bytes[offset + 4] = limb_bytes[4];
        bytes[offset + 5] = limb_bytes[5];
        bytes[offset + 6] = limb_bytes[6];
        bytes[offset + 7] = limb_bytes[7];
    }
    bytes
}

/// Returns true if the big-endian byte representation is less than the BN254
/// scalar field (Fr) modulus. Use this to reject non-canonical public inputs
/// before converting to Fr.
pub fn is_less_than_bn254_field_size_be(bytes: &[u8; 32]) -> bool {
    let value = bigint_from_be_bytes(bytes);
    value < <ArkFr as PrimeField>::MODULUS
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

    /// Convert big-endian bytes to Fr, rejecting non-canonical values.
    /// Returns `None` if the value is >= the BN254 scalar field modulus.
    pub fn from_be_bytes(bytes: &[u8; 32]) -> Option<Self> {
        if !is_less_than_bn254_field_size_be(bytes) {
            return None;
        }
        Some(Self::from_be_bytes_unchecked(bytes))
    }

    /// Convert big-endian bytes to Fr with silent modular reduction.
    /// Non-canonical values (>= field modulus) are reduced without error.
    pub fn from_be_bytes_unchecked(bytes: &[u8]) -> Self {
        let mut le = [0u8; 32];
        let len = bytes.len().min(32);
        let start = 32 - len;
        le[start..].copy_from_slice(&bytes[..len]);
        le.reverse();
        Self(ArkFr::from_le_bytes_mod_order(&le))
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        bigint_to_be_bytes(&self.0.into_bigint())
    }

    pub fn square(&self) -> Self {
        Self(self.0.square())
    }

    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(Self)
    }
}

impl core::ops::Add for Fr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl core::ops::Sub for Fr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl core::ops::Mul for Fr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl core::ops::Neg for Fr {
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
    fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Fr {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Fr::from_be_bytes(&buf).ok_or_else(|| {
            borsh::io::Error::new(
                borsh::io::ErrorKind::InvalidData,
                "Fr value >= field modulus",
            )
        })
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
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("32 bytes")
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Fr, E> {
                let bytes: [u8; 32] = v
                    .try_into()
                    .map_err(|_| E::invalid_length(v.len(), &"32 bytes"))?;
                Fr::from_be_bytes(&bytes).ok_or_else(|| E::custom("Fr value >= field modulus"))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Fr, A::Error> {
                let mut bytes = [0u8; 32];
                for (i, b) in bytes.iter_mut().enumerate() {
                    *b = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &"32 bytes"))?;
                }
                Fr::from_be_bytes(&bytes)
                    .ok_or_else(|| serde::de::Error::custom("Fr value >= field modulus"))
            }
        }
        deserializer.deserialize_bytes(FrVisitor)
    }
}
