#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
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

# --- Compile circuit ---
echo "=== Compiling circom circuit (--O1 for PLONK) ==="
circom circuits/multiplier.circom --r1cs --wasm --sym -o build --O1

# --- PLONK setup (no phase 2 ceremony needed) ---
echo "=== Running PLONK setup ==="
npx snarkjs plonk setup \
    build/multiplier.r1cs \
    "$PTAU_FILE" \
    build/multiplier.zkey

# --- Export verification key ---
echo "=== Exporting verification key ==="
npx snarkjs zkey export verificationkey \
    build/multiplier.zkey \
    build/verification_key.json

# --- Generate sample proof ---
echo "=== Generating sample proof (a=3, b=11, c=33) ==="
echo '{"a": "3", "b": "11"}' > build/input.json

npx snarkjs plonk fullprove \
    build/input.json \
    build/multiplier_js/multiplier.wasm \
    build/multiplier.zkey \
    build/proof.json \
    build/public.json

# --- Verify proof with snarkjs (sanity check) ---
echo "=== Verifying proof with snarkjs ==="
npx snarkjs plonk verify \
    build/verification_key.json \
    build/public.json \
    build/proof.json

echo ""
echo "=== Setup complete ==="
echo "Artifacts in build/:"
echo "  - verification_key.json"
echo "  - proof.json"
echo "  - public.json"
echo "  - multiplier.zkey"
