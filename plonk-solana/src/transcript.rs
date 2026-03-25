/// Keccak256 transcript matching snarkjs Keccak256Transcript.
///
/// Uses light-hasher for keccak256 (Solana syscall on-chain, sha3 off-chain).
///
/// Format:
/// - G1 points: 64 bytes (x || y), big-endian
/// - Scalars: 32 bytes, big-endian
/// - Challenge: keccak256(buffer) -> reduce mod scalar field order
use alloc::vec::Vec;

use crate::errors::PlonkError;
use crate::fr::Fr;
use crate::g1::G1;
use light_hasher::{Hasher, Keccak};

pub struct Transcript {
    data: Vec<u8>,
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

impl Transcript {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn add_point(&mut self, p: &G1) {
        self.data.extend_from_slice(&p.0);
    }

    pub fn add_scalar(&mut self, s: &Fr) {
        self.data.extend_from_slice(&s.to_be_bytes());
    }

    pub fn get_challenge(&self) -> Result<Fr, PlonkError> {
        if self.data.is_empty() {
            return Err(PlonkError::EmptyTranscript);
        }
        let hash = Keccak::hash(&self.data).map_err(|_| PlonkError::KeccakFailed)?;
        Ok(Fr::from_be_bytes_unchecked(&hash))
    }
}
