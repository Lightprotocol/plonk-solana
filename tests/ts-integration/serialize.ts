/**
 * Serialization helpers for converting snarkjs PLONK proof output
 * into the binary instruction data format expected by the on-chain program.
 *
 * Instruction data format:
 *   [0]               -> u8: number of public inputs (N)
 *   [1..1+N*32]       -> N x 32 bytes: public inputs (big-endian Fr)
 *   [1+N*32..+9*64]   -> 9 x 64 bytes: G1 proof points (uncompressed BE x||y)
 *   [+9*64..+6*32]    -> 6 x 32 bytes: Fr field evaluations (big-endian)
 */

export interface SnarkjsPlonkProof {
  A: string[];
  B: string[];
  C: string[];
  Z: string[];
  T1: string[];
  T2: string[];
  T3: string[];
  Wxi: string[];
  Wxiw: string[];
  eval_a: string;
  eval_b: string;
  eval_c: string;
  eval_s1: string;
  eval_s2: string;
  eval_zw: string;
  protocol: string;
  curve: string;
}

/** Convert a decimal BigInt string to a 32-byte big-endian Uint8Array. */
export function bigintToBeBytes(value: string): Uint8Array {
  let n = BigInt(value);
  const buf = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) {
    buf[i] = Number(n & 0xffn);
    n >>= 8n;
  }
  return buf;
}

/**
 * Serialize a snarkjs G1 point (Jacobian coords with z=1, i.e. affine)
 * to 64-byte uncompressed format: x_be_32 || y_be_32.
 */
export function serializeG1(coords: string[]): Uint8Array {
  if (coords[2] !== "1") {
    throw new Error(
      `Expected affine G1 point (z=1), got z=${coords[2]}. ` +
        "snarkjs should normalize to affine in JSON output.",
    );
  }
  const buf = new Uint8Array(64);
  buf.set(bigintToBeBytes(coords[0]), 0);
  buf.set(bigintToBeBytes(coords[1]), 32);
  return buf;
}

/**
 * Build the full instruction data buffer from a snarkjs proof and public signals.
 * Matches the on-chain parsing in integration-program/src/lib.rs.
 */
export function buildInstructionData(
  proof: SnarkjsPlonkProof,
  publicSignals: string[],
): Buffer {
  const nPublic = publicSignals.length;
  const totalLen = 1 + nPublic * 32 + 9 * 64 + 6 * 32;
  const buf = Buffer.alloc(totalLen);
  let offset = 0;

  // Number of public inputs
  buf[offset++] = nPublic;

  // Public inputs (big-endian Fr)
  for (const signal of publicSignals) {
    buf.set(bigintToBeBytes(signal), offset);
    offset += 32;
  }

  // 9 G1 proof points in order: A, B, C, Z, T1, T2, T3, Wxi, Wxiw
  const g1Points = [
    proof.A,
    proof.B,
    proof.C,
    proof.Z,
    proof.T1,
    proof.T2,
    proof.T3,
    proof.Wxi,
    proof.Wxiw,
  ];
  for (const point of g1Points) {
    buf.set(serializeG1(point), offset);
    offset += 64;
  }

  // 6 Fr evaluations in order: eval_a, eval_b, eval_c, eval_s1, eval_s2, eval_zw
  const evals = [
    proof.eval_a,
    proof.eval_b,
    proof.eval_c,
    proof.eval_s1,
    proof.eval_s2,
    proof.eval_zw,
  ];
  for (const val of evals) {
    buf.set(bigintToBeBytes(val), offset);
    offset += 32;
  }

  return buf;
}
