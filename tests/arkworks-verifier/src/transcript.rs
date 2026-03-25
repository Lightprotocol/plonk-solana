/// Keccak256 transcript matching snarkjs Keccak256Transcript.
///
/// Serialization format:
/// - G1 points: uncompressed affine (x || y), each coordinate 32 bytes big-endian
/// - Scalars (Fr): 32 bytes big-endian
/// - Challenge: keccak256(buffer) interpreted as big-endian integer, reduced mod r
use ark_bn254::{Fq, Fr, G1Affine};
use num_bigint::BigUint;
use sha3::{Digest, Keccak256};

enum TranscriptEntry {
    Point(G1Affine),
    Scalar(Fr),
}

pub struct Transcript {
    data: Vec<TranscriptEntry>,
}

/// Serialize an Fq element to 32 bytes big-endian.
fn fq_to_be_bytes(f: &Fq) -> [u8; 32] {
    let n: BigUint = (*f).into();
    let bytes = n.to_bytes_be();
    let mut result = [0u8; 32];
    let offset = 32 - bytes.len();
    result[offset..].copy_from_slice(&bytes);
    result
}

/// Serialize an Fr element to 32 bytes big-endian.
fn fr_to_be_bytes(f: &Fr) -> [u8; 32] {
    let n: BigUint = (*f).into();
    let bytes = n.to_bytes_be();
    let mut result = [0u8; 32];
    let offset = 32 - bytes.len();
    result[offset..].copy_from_slice(&bytes);
    result
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

    pub fn add_point(&mut self, p: &G1Affine) {
        self.data.push(TranscriptEntry::Point(*p));
    }

    pub fn add_scalar(&mut self, s: &Fr) {
        self.data.push(TranscriptEntry::Scalar(*s));
    }

    /// Compute challenge: keccak256 of all entries, reduced mod r.
    pub fn get_challenge(&self) -> Fr {
        assert!(!self.data.is_empty(), "transcript has no data");

        // Calculate buffer size
        let mut size = 0;
        for entry in &self.data {
            match entry {
                TranscriptEntry::Point(_) => size += 64, // x(32) + y(32)
                TranscriptEntry::Scalar(_) => size += 32,
            }
        }

        let mut buffer = vec![0u8; size];
        let mut offset = 0;

        for entry in &self.data {
            match entry {
                TranscriptEntry::Point(p) => {
                    let x_bytes = fq_to_be_bytes(&p.x);
                    let y_bytes = fq_to_be_bytes(&p.y);
                    buffer[offset..offset + 32].copy_from_slice(&x_bytes);
                    buffer[offset + 32..offset + 64].copy_from_slice(&y_bytes);
                    offset += 64;
                }
                TranscriptEntry::Scalar(s) => {
                    let s_bytes = fr_to_be_bytes(s);
                    buffer[offset..offset + 32].copy_from_slice(&s_bytes);
                    offset += 32;
                }
            }
        }

        let hash = Keccak256::digest(&buffer);
        let value = BigUint::from_bytes_be(&hash);
        Fr::from(value)
    }
}
