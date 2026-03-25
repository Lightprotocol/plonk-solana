//! # plonk-solana
//!
//! PLONK zero-knowledge proof verification for Solana using alt_bn128 syscalls.
//!
//! **Experimental and unaudited. Do not use in production.**
//!
//! Compatible with [snarkjs](https://github.com/iden3/snarkjs) PLONK proofs
//! over the BN254 curve (circom circuits compiled with `--O1`).
//!
//! All inputs are big-endian byte arrays matching the EIP-197 / snarkjs format.
//!
//! ## Types
//!
//! - [`G1`] / [`CompressedG1`] -- uncompressed (64 bytes) and compressed (32 bytes) BN254 G1 points
//! - [`G2`] -- BN254 G2 point (128 bytes, EIP-197 order)
//! - [`Fr`] -- BN254 scalar field element
//! - [`Proof`] / [`CompressedProof`] -- 9 G1 commitments + 6 scalar evaluations
//! - [`VerificationKey`] -- selector/permutation commitments + domain parameters
//!
//! ## Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `bytemuck` | `Pod`, `Zeroable` for G1, CompressedG1, G2 |
//! | `zerocopy` | `FromBytes`, `IntoBytes`, etc. for G1, CompressedG1, G2 |
//! | `borsh` | `BorshSerialize`, `BorshDeserialize` for all types |
//! | `serde` | `Serialize`, `Deserialize` for all types |
//! | `vk` | Verification key JSON parser (enables `serde`) |
//!
//! ## Usage
//!
//! The verification key is embedded at compile time.
//! The proof and public inputs arrive in instruction data.
//!
//! ```rust,no_run
//! use plonk_solana::{verify, CompressedProof, VerificationKey};
//!
//! # fn verifying_key() -> VerificationKey { unimplemented!() }
//! # fn compressed_proof() -> CompressedProof { unimplemented!() }
//! // Baked into the program at compile time (see vk_parser::generate_vk_file)
//! let vk: VerificationKey = verifying_key();
//!
//! // Deserialized from instruction data
//! let compressed_proof: CompressedProof = compressed_proof();
//! let public_input_1: [u8; 32] = [0u8; 32];
//! let public_input_2: [u8; 32] = [0u8; 32];
//!
//! let proof = compressed_proof.decompress()?;
//! verify(&vk, &proof, &[public_input_1, public_input_2])?;
//! # Ok::<(), plonk_solana::PlonkError>(())
//! ```

#![no_std]

extern crate alloc;

#[cfg(feature = "bench")]
pub mod errors;
#[cfg(not(feature = "bench"))]
pub(crate) mod errors;

#[cfg(feature = "bench")]
pub mod fr;
#[cfg(not(feature = "bench"))]
pub(crate) mod fr;

#[cfg(feature = "bench")]
pub mod g1;
#[cfg(not(feature = "bench"))]
pub(crate) mod g1;

#[cfg(feature = "bench")]
pub mod g2;
#[cfg(not(feature = "bench"))]
pub(crate) mod g2;

#[cfg(feature = "bench")]
pub mod plonk;
#[cfg(not(feature = "bench"))]
pub(crate) mod plonk;

pub mod syscalls;

#[cfg(feature = "bench")]
pub mod transcript;
#[cfg(not(feature = "bench"))]
pub(crate) mod transcript;

#[cfg(any(feature = "vk", test))]
pub mod vk_parser;

pub use errors::PlonkError;
pub use fr::{is_less_than_bn254_field_size_be, Fr};
pub use g1::{CompressedG1, G1};
pub use g2::G2;
pub use plonk::{verify, verify_unchecked, CompressedProof, Proof, VerificationKey};
