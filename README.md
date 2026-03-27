<!-- cargo-rdme start -->

## plonk-solana

PLONK zero-knowledge proof verification for Solana using alt_bn128 syscalls.

**Experimental and unaudited. Do not use in production.**

Compatible with [snarkjs](https://github.com/iden3/snarkjs) PLONK proofs
over the BN254 curve (circom circuits compiled with `--O1`).

All inputs are big-endian byte arrays matching the EIP-197 / snarkjs format.

### Types

- [`G1`] / [`CompressedG1`] -- uncompressed (64 bytes) and compressed (32 bytes) BN254 G1 points
- [`G2`] -- BN254 G2 point (128 bytes, EIP-197 order)
- [`Fr`] -- BN254 scalar field element
- [`Proof`] / [`CompressedProof`] -- 9 G1 commitments + 6 scalar evaluations
- [`VerificationKey`] -- selector/permutation commitments + domain parameters

### Feature flags

| Feature | Description |
|---------|-------------|
| `bytemuck` | `Pod`, `Zeroable` for G1, CompressedG1, G2 |
| `zerocopy` | `FromBytes`, `IntoBytes`, etc. for G1, CompressedG1, G2 |
| `borsh` | `BorshSerialize`, `BorshDeserialize` for all types |
| `serde` | `Serialize`, `Deserialize` for all types |
| `vk` | Verification key JSON parser (enables `serde`) |

### Usage

The verification key is embedded at compile time.
The proof and public inputs arrive in instruction data.

```rust
use plonk_solana::{verify, CompressedProof, VerificationKey};

// Baked into the program at compile time (see vk_parser::generate_vk_file)
let vk: VerificationKey = verifying_key();

// Deserialized from instruction data
let compressed_proof: CompressedProof = compressed_proof();
let public_input_1: [u8; 32] = [0u8; 32];
let public_input_2: [u8; 32] = [0u8; 32];

let proof = compressed_proof.decompress()?;
verify(&vk, &proof, &[public_input_1, public_input_2])?;
```

<!-- cargo-rdme end -->

## Benchmarks

See [BENCHMARKS.md](BENCHMARKS.md) for detailed per-function CU profiling.

| Public Inputs | `verify` CU | `verify_unchecked` CU |
|:---:|---:|---:|
| 1 | 322,782 | 320,316 |

*Measured on Solana SBF with [light-program-profiler](https://github.com/Lightprotocol/light-program-profiler). Run `just bench` to regenerate.*
