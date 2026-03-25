#![cfg(feature = "vk")]

mod common;

use plonk_solana::{CompressedG1, Fr, G1, G2};

#[cfg(feature = "borsh")]
mod borsh_tests {
    use super::*;

    #[test]
    fn test_roundtrip_g1() {
        let point = G1::GENERATOR;
        let encoded = borsh::to_vec(&point).unwrap();
        assert_eq!(encoded.len(), 64, "G1 borsh encoding should be 64 bytes");
        let decoded: G1 = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, point, "G1 borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_g1_zero() {
        let point = G1::ZERO;
        let encoded = borsh::to_vec(&point).unwrap();
        let decoded: G1 = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, point, "G1::ZERO borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_compressed_g1() {
        let compressed = G1::GENERATOR.compress().unwrap();
        let encoded = borsh::to_vec(&compressed).unwrap();
        assert_eq!(
            encoded.len(),
            32,
            "CompressedG1 borsh encoding should be 32 bytes"
        );
        let decoded: CompressedG1 = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, compressed, "CompressedG1 borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_g2() {
        let point = G2::GENERATOR;
        let encoded = borsh::to_vec(&point).unwrap();
        assert_eq!(encoded.len(), 128, "G2 borsh encoding should be 128 bytes");
        let decoded: G2 = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, point, "G2 borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_fr() {
        let scalar = Fr::from(33u64);
        let encoded = borsh::to_vec(&scalar).unwrap();
        assert_eq!(encoded.len(), 32, "Fr borsh encoding should be 32 bytes");
        let decoded: Fr = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, scalar, "Fr borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_fr_zero() {
        let scalar = Fr::zero();
        let encoded = borsh::to_vec(&scalar).unwrap();
        let decoded: Fr = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, scalar, "Fr::zero() borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_proof() {
        let proof = common::load_test_proof();
        let encoded = borsh::to_vec(&proof).unwrap();
        let decoded: plonk_solana::Proof = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, proof, "Proof borsh roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_compressed_proof() {
        let proof = common::load_test_proof();
        let compressed = proof.compress().unwrap();
        let encoded = borsh::to_vec(&compressed).unwrap();
        let decoded: plonk_solana::CompressedProof = borsh::from_slice(&encoded).unwrap();
        assert_eq!(
            decoded, compressed,
            "CompressedProof borsh roundtrip mismatch"
        );
    }

    #[test]
    fn test_roundtrip_verification_key() {
        let vk = common::load_test_vk();
        let encoded = borsh::to_vec(&vk).unwrap();
        let decoded: plonk_solana::VerificationKey = borsh::from_slice(&encoded).unwrap();
        assert_eq!(decoded, vk, "VerificationKey borsh roundtrip mismatch");
    }
}

mod serde_tests {
    use super::*;

    #[test]
    fn test_roundtrip_g1() {
        let point = G1::GENERATOR;
        let encoded = serde_json::to_vec(&point).unwrap();
        let decoded: G1 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, point, "G1 serde roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_compressed_g1() {
        let compressed = G1::GENERATOR.compress().unwrap();
        let encoded = serde_json::to_vec(&compressed).unwrap();
        let decoded: CompressedG1 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, compressed, "CompressedG1 serde roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_g2() {
        let point = G2::GENERATOR;
        let encoded = serde_json::to_vec(&point).unwrap();
        let decoded: G2 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, point, "G2 serde roundtrip mismatch");
    }

    #[test]
    fn test_roundtrip_fr() {
        let scalar = Fr::from(33u64);
        let encoded = serde_json::to_vec(&scalar).unwrap();
        let decoded: Fr = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, scalar, "Fr serde roundtrip mismatch");
    }
}
