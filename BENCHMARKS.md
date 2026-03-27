# PLONK Verifier CU Benchmarks

Compute unit benchmarks for PLONK verification on Solana (1 public input).

All CU values have baseline profiling overhead (7 CU) subtracted.

## External API

| Function | CU |
|----------|-----|
| [bench_verify](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L6) | 322,782 |
| [bench_verify_unchecked](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L15) | 320,316 |
| [bench_proof_compress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L24) | 1,504 |
| [bench_proof_decompress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/top_level.rs#L29) | 5,056 |

## Internal API

### Fr Ops

| Function | CU |
|----------|-----|
| [bench_fr_from_be_bytes](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L5) | 2,455 |
| [bench_fr_to_be_bytes](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L10) | 1,202 |
| [bench_fr_square](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L15) | 1,978 |
| [bench_fr_inverse](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L20) | 56,150 |
| [bench_fr_add](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L25) | 53 |
| [bench_fr_sub](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L30) | 85 |
| [bench_fr_mul](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L35) | 2,395 |
| [bench_is_less_than_field_size](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/fr_ops.rs#L40) | 10 |

### G1 Ops

| Function | CU |
|----------|-----|
| [bench_g1_add](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L8) | 420 |
| [bench_g1_neg](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L13) | 87 |
| [bench_g1_mul](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L18) | 5,098 |
| [bench_g1_compress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L23) | 158 |
| [bench_g1_decompress](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/g1_ops.rs#L28) | 543 |

### Transcript Ops

| Function | CU |
|----------|-----|
| [bench_transcript_get_challenge](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/transcript_ops.rs#L6) | 3,834 |
| [bench_calculate_challenges](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/transcript_ops.rs#L13) | 38,406 |

### Verification Ops

| Function | CU |
|----------|-----|
| [bench_calculate_l1_and_pi](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L7) | 66,258 |
| [bench_calculate_r0_and_d](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L16) | 92,297 |
| [bench_calculate_f](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L28) | 27,635 |
| [bench_is_valid_pairing](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/src/benchmarks/verification_ops.rs#L38) | 88,403 |

