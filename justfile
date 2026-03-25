# plonk-solana justfile

# Run all tests (unit + integration)
test: test-unit test-integration

# Unit tests for plonk-solana crate
test-unit:
    cargo test -p plonk-solana

# Build SBF program and run litesvm integration tests
test-integration: build-sbf
    cargo test-sbf --manifest-path tests/integration-program/Cargo.toml

# Build the integration program for SBF
build-sbf:
    cargo build-sbf --manifest-path tests/integration-program/Cargo.toml

# Run arkworks reference verifier tests
test-arkworks:
    cargo test -p plonk-verifier

# Check all crates compile
check:
    cargo check --workspace

# Format all Rust code
format:
    cargo fmt --all

# Check formatting without modifying files
format-check:
    cargo fmt --all -- --check

# Lint all Rust code (all features enabled)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

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
