# plonk-solana

Experiment to build a PLONK proof verifier for Solana, similar to groth16-solana but for the PLONK proof system.

## Project Structure

- `plonk-solana/` - Main PLONK verifier library (solana-bn254 syscalls)
- `tests/arkworks-verifier/` - Pure Rust PLONK verifier using arkworks (off-chain reference)
- `tests/integration-program/` - Solana on-chain verifier with LiteSVM tests
- `tests/fixtures/circuits/` - Circom circuit definitions (compile with `--O1` for PLONK)
- `tests/fixtures/scripts/` - Circuit build and clean scripts
- `tests/fixtures/data/` - Shared test artifacts (verification_key.json, proof.json, public.json)

## Setup

```bash
just build-test-circuit   # compile circuit, PLONK setup, generate sample proof
cargo test -p plonk-solana            # test main verifier
cargo test -p plonk-verifier          # test arkworks reference verifier
```

## Current State

- Minimal circom multiplier circuit (a * b = c)
- snarkjs PLONK proof generation (`just build-test-circuit`)
- Two independent Rust PLONK verifiers that both pass:
  1. `tests/arkworks-verifier/` - arkworks-based, pure Rust
  2. `plonk-solana/` - solana-bn254 syscall-based, Solana-compatible
- On-chain integration program with LiteSVM tests (`tests/integration-program/`)

## Future Work

- Measure compute unit usage on-chain
- Optimize for CU efficiency (reduce G1 scalar multiplications)
- Consider removing arkworks dependency from on-chain code for smaller binary size

## PLONK-Specific Notes

- Compile circom with `--O1` (not `--O2`). PLONK does not support full simplification.
- PLONK setup does not require a phase 2 ceremony (unlike Groth16).
- Transcript uses Keccak256 (matching snarkjs Keccak256Transcript).
- Verification key contains selector polynomial commitments (Qm, Ql, Qr, Qo, Qc) and permutation polynomial commitments (S1, S2, S3).
- Proof contains 9 G1 points + 6 field evaluations.
- Verification: 5 challenge rounds, Lagrange evaluation, linearization, 2-pair pairing check.

## Key Dependencies

- `solana-bn254` - BN254 curve operations via Solana syscalls
- `light-hasher` - Keccak256 (Solana syscall on-chain, sha3 off-chain)
- `ark-bn254` / `ark-ff` - Field arithmetic for Fr
- `snarkjs` (npm) - PLONK proof generation

## Testing

- Main verifier: `cargo test -p plonk-solana`
- Arkworks reference: `cargo test -p plonk-verifier`
- On-chain integration: `cargo test-sbf --manifest-path tests/integration-program/Cargo.toml`
- To regenerate proof artifacts: `just build-test-circuit` (requires node, npm, circom)
