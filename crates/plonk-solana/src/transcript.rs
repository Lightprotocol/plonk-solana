/// Keccak256 transcript matching snarkjs Keccak256Transcript.
///
/// Uses light-hasher for keccak256 (Solana syscall on-chain, sha3 off-chain).
///
/// Format:
/// - G1 points: 64 bytes (x || y), big-endian
/// - Scalars: 32 bytes, big-endian
/// - Challenge: keccak256(buffer) -> reduce mod scalar field order
use crate::errors::PlonkError;
use crate::fr::Fr;
use light_hasher::{Hasher, Keccak};

enum Entry {
    Point([u8; 64]),
    Scalar([u8; 32]),
}

pub struct Transcript {
    data: Vec<Entry>,
}

impl Transcript {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn add_point(&mut self, p: &[u8; 64]) {
        self.data.push(Entry::Point(*p));
    }

    pub fn add_scalar(&mut self, s: &Fr) {
        self.data.push(Entry::Scalar(s.to_be_bytes()));
    }

    pub fn get_challenge(&self) -> Result<Fr, PlonkError> {
        let mut size = 0;
        for entry in &self.data {
            match entry {
                Entry::Point(_) => size += 64,
                Entry::Scalar(_) => size += 32,
            }
        }

        let mut buffer = vec![0u8; size];
        let mut offset = 0;
        for entry in &self.data {
            match entry {
                Entry::Point(p) => {
                    buffer[offset..offset + 64].copy_from_slice(p);
                    offset += 64;
                }
                Entry::Scalar(s) => {
                    buffer[offset..offset + 32].copy_from_slice(s);
                    offset += 32;
                }
            }
        }

        let hash = Keccak::hash(&buffer).map_err(|_| PlonkError::KeccakFailed)?;
        Ok(Fr::from_be_bytes(&hash))
    }
}
