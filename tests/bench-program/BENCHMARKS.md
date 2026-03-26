# PLONK Verifier CU Benchmarks

Compute unit benchmarks for PLONK verification operations on Solana.

## Table of Contents

**[1. Baseline](#1-baseline)**

**[2. Benchmarks](#2-benchmarks)**

  - [2.1 Fr Ops](#21-fr-ops)
  - [2.2 G1 Ops](#22-g1-ops)
  - [2.3 Top Level](#23-top-level)
  - [2.4 Transcript Ops](#24-transcript-ops)
  - [2.5 Verification Ops](#25-verification-ops)


## Definitions

- **CU**: Compute units consumed by the operation (baseline profiling overhead of 7 CU subtracted)

## 1. Baseline

### 1.1 Lib

| Function | CU Consumed | CU |
|----------|-------------|-----|
| [baseline_empty_function](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/lib.rs#L15) | 7 | 0 |

## 2. Benchmarks

### 2.1 Fr Ops

| Function | CU |
|----------|-----|
| [bench_fr_from_be_bytes](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L5) | 2455 |
| [bench_fr_to_be_bytes](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L10) | 1202 |
| [bench_fr_square](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L15) | 1978 |
| [bench_fr_inverse](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L20) | 56150 |
| [bench_fr_add](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L25) | 80 |
| [bench_fr_sub](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L30) | 54 |
| [bench_fr_mul](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L35) | 2397 |
| [bench_is_less_than_field_size](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L40) | 10 |

### 2.2 G1 Ops

| Function | CU |
|----------|-----|
| [bench_g1_add](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L8) | 420 |
| [bench_g1_neg](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L13) | 88 |
| [bench_g1_mul](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L18) | 5101 |
| [bench_g1_compress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L23) | 158 |
| [bench_g1_decompress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L28) | 543 |

### 2.3 Top Level

| Function | CU |
|----------|-----|
| [bench_verify](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L6) | 322199 |
| [bench_verify_unchecked](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L15) | 319733 |
| [bench_proof_compress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L24) | 1504 |
| [bench_proof_decompress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L29) | 5056 |

### 2.4 Transcript Ops

| Function | CU |
|----------|-----|
| [bench_transcript_get_challenge](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/transcript_ops.rs#L6) | 3834 |
| [bench_calculate_challenges](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/transcript_ops.rs#L13) | 38649 |

### 2.5 Verification Ops

| Function | CU |
|----------|-----|
| [bench_calculate_l1_and_pi](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L7) | 65354 |
| [bench_calculate_r0_and_d](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L16) | 92362 |
| [bench_calculate_f](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L28) | 27631 |
| [bench_is_valid_pairing](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L38) | 88415 |

