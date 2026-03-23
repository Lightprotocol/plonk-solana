# plonk-solana

Experiment to build a PLONK proof verifier for Solana, similar to groth16-solana but for the PLONK proof system.

## Project Structure

- `circuits/` - Circom circuit definitions (compile with `--O1` for PLONK)
- `scripts/` - Setup and clean scripts
- `verifier/` - Pure Rust PLONK verifier using arkworks (off-chain reference)
- `programs/plonk-verifier/` - Solana-compatible PLONK verifier using solana-bn254 syscalls

## Setup

```bash
npm run setup    # compile circuit, PLONK setup, generate sample proof
cargo test -p plonk-verifier          # test off-chain verifier
cargo test -p plonk-verifier-program  # test on-chain verifier
```

## Current State

- Minimal circom multiplier circuit (a * b = c)
- snarkjs PLONK proof generation (setup script)
- Two independent Rust PLONK verifiers that both pass:
  1. `verifier/` - arkworks-based, pure Rust
  2. `programs/plonk-verifier/` - solana-bn254 syscall-based, Solana-compatible
- The Solana program entry point is not yet wired up (no process_instruction)

## Future Work

- Add Solana program entry point (process_instruction) to programs/plonk-verifier
- Deploy and test on localnet with cargo test-sbf
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

- Off-chain: `cargo test -p plonk-verifier`
- On-chain lib: `cargo test -p plonk-verifier-program`
- Proof artifacts must exist in `build/` (run `npm run setup` first)
