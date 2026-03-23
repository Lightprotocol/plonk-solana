/// Thin wrapper around ark_bn254::Fr for convenient byte conversion.
/// All serialization uses 32-byte big-endian format (matching snarkjs/EIP-197).
use ark_bn254::Fr as ArkFr;
use ark_ff::{Field, One, Zero};
use num_bigint::BigUint;

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
