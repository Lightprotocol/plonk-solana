# plonk-solana justfile

# Run all tests (unit + arkworks + integration + TypeScript)
test: test-unit test-arkworks test-integration test-ts

# Unit tests for plonk-solana crate
test-unit:
    cargo test -p plonk-solana

# Build SBF program and run litesvm integration tests
test-integration: build-sbf
    cargo test-sbf --manifest-path tests/integration-program/Cargo.toml

# Build the integration program for SBF
build-sbf:
    cargo build-sbf --manifest-path tests/integration-program/Cargo.toml

# TypeScript integration test (dynamic snarkjs proof generation)
test-ts: build-sbf
    SBF_OUT_DIR=target/deploy npx vitest run --config tests/ts-integration/vitest.config.ts

# Run arkworks reference verifier tests
test-arkworks:
    cargo test -p plonk-verifier

# Check all crates compile
check:
    cargo check --workspace

# Format all Rust code and regenerate README
format:
    cargo fmt --all
    cargo rdme --force

# Check formatting and README without modifying files
format-check:
    cargo fmt --all -- --check
    cargo rdme --check --force

# Lint all Rust code (all features enabled) and check README
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo check -p plonk-solana --no-default-features
    cargo rdme --check --force

# Format + lint (fix all issues)
fix: format
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty -- -D warnings

# Compile circom circuit + generate PLONK proof (requires node, npm, circom)
build-test-circuit:
    ./tests/fixtures/scripts/setup.sh

# Remove build artifacts
clean:
    ./tests/fixtures/scripts/clean.sh
    cargo clean
