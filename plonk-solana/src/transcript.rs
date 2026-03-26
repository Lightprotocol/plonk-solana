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

/// Hash multiple byte slices as a single keccak256 input and return an Fr challenge.
pub fn hash_challenge(slices: &[&[u8]]) -> Result<Fr, PlonkError> {
    let hash = Keccak::hashv(slices).map_err(|_| PlonkError::KeccakFailed)?;
    Ok(Fr::from_be_bytes_unchecked(&hash))
}
