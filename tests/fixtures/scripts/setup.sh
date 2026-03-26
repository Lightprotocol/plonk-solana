#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$ROOT_DIR"

# --- Check dependencies ---
for cmd in node npm circom; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "ERROR: $cmd is not installed"
        exit 1
    fi
done

echo "=== Installing npm dependencies ==="
npm install

echo "=== Creating output directories ==="
mkdir -p build pot

# --- Download Powers of Tau ---
PTAU_FILE="pot/powersOfTau28_hez_final_08.ptau"
if [ ! -f "$PTAU_FILE" ]; then
    echo "=== Downloading Powers of Tau (2^8) ==="
    curl -L -o "$PTAU_FILE" \
        "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_08.ptau"
else
    echo "=== Powers of Tau already downloaded ==="
fi

# --- Build original multiplier circuit (backward compat) ---
echo "=== Compiling multiplier circuit (--O1 for PLONK) ==="
circom tests/fixtures/circuits/multiplier.circom --r1cs --wasm --sym -o build --O1

echo "=== Running PLONK setup for multiplier ==="
npx snarkjs plonk setup \
    build/multiplier.r1cs \
    "$PTAU_FILE" \
    build/multiplier.zkey

echo "=== Exporting verification key ==="
npx snarkjs zkey export verificationkey \
    build/multiplier.zkey \
    build/verification_key.json

echo "=== Generating sample proof (a=3, b=11, c=33) ==="
echo '{"a": "3", "b": "11"}' > build/input.json

npx snarkjs plonk fullprove \
    build/input.json \
    build/multiplier_js/multiplier.wasm \
    build/multiplier.zkey \
    build/proof.json \
    build/public.json

echo "=== Verifying proof with snarkjs ==="
npx snarkjs plonk verify \
    build/verification_key.json \
    build/public.json \
    build/proof.json

echo "=== Copying multiplier artifacts to test fixtures ==="
cp build/verification_key.json tests/fixtures/data/
cp build/proof.json tests/fixtures/data/
cp build/public.json tests/fixtures/data/

# --- Build mul1..mul5 circuits (varying public output counts) ---
INPUT_JSON='{"a": "3", "b": "11"}'

for n in 1 2 3 4 5; do
    CIRCUIT="mul${n}"
    echo ""
    echo "=== Building ${CIRCUIT} circuit (${n} public outputs) ==="

    circom "tests/fixtures/circuits/${CIRCUIT}.circom" --r1cs --wasm --sym -o build --O1

    npx snarkjs plonk setup \
        "build/${CIRCUIT}.r1cs" \
        "$PTAU_FILE" \
        "build/${CIRCUIT}.zkey"

    npx snarkjs zkey export verificationkey \
        "build/${CIRCUIT}.zkey" \
        "build/${CIRCUIT}_verification_key.json"

    echo "$INPUT_JSON" > "build/${CIRCUIT}_input.json"

    npx snarkjs plonk fullprove \
        "build/${CIRCUIT}_input.json" \
        "build/${CIRCUIT}_js/${CIRCUIT}.wasm" \
        "build/${CIRCUIT}.zkey" \
        "build/${CIRCUIT}_proof.json" \
        "build/${CIRCUIT}_public.json"

    npx snarkjs plonk verify \
        "build/${CIRCUIT}_verification_key.json" \
        "build/${CIRCUIT}_public.json" \
        "build/${CIRCUIT}_proof.json"

    mkdir -p "tests/fixtures/data/${CIRCUIT}"
    cp "build/${CIRCUIT}_verification_key.json" "tests/fixtures/data/${CIRCUIT}/verification_key.json"
    cp "build/${CIRCUIT}_proof.json" "tests/fixtures/data/${CIRCUIT}/proof.json"
    cp "build/${CIRCUIT}_public.json" "tests/fixtures/data/${CIRCUIT}/public.json"

    echo "  ${CIRCUIT}: OK"
done

echo ""
echo "=== Setup complete ==="
echo "Artifacts in build/ and tests/fixtures/data/"
