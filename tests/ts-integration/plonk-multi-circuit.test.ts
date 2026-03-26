import { describe, it, expect } from "vitest";
import * as fs from "fs";
// @ts-ignore -- snarkjs has no types
import * as snarkjs from "snarkjs";
import path from "path";

const PROJECT_ROOT = path.resolve(__dirname, "../..");

const circuits = [
  { name: "mul1", nOutputs: 1, expected: ["33"] },
  { name: "mul2", nOutputs: 2, expected: ["33", "99"] },
  { name: "mul3", nOutputs: 3, expected: ["33", "99", "1089"] },
  { name: "mul4", nOutputs: 4, expected: ["33", "99", "1089", "3267"] },
  { name: "mul5", nOutputs: 5, expected: ["33", "99", "1089", "3267", "35937"] },
];

describe.each(circuits)(
  "$name circuit ($nOutputs public outputs)",
  ({ name, nOutputs, expected }) => {
    const wasmPath = path.join(
      PROJECT_ROOT,
      `build/${name}_js/${name}.wasm`,
    );
    const zkeyPath = path.join(PROJECT_ROOT, `build/${name}.zkey`);
    const vkPath = path.join(
      PROJECT_ROOT,
      `tests/fixtures/data/${name}/verification_key.json`,
    );

    it("generates proof with correct public signals", async () => {
      const { proof, publicSignals } = await snarkjs.plonk.fullProve(
        { a: "3", b: "11" },
        wasmPath,
        zkeyPath,
      );

      expect(publicSignals).toEqual(expected);
      expect(publicSignals).toHaveLength(nOutputs);

      // Verify with snarkjs
      const vk = JSON.parse(fs.readFileSync(vkPath, "utf8"));
      const valid = await snarkjs.plonk.verify(vk, publicSignals, proof);
      expect(valid).toBe(true);
    });

    it("generates proof with different inputs (a=7, b=13)", async () => {
      // a=7, b=13: c1=91, c2=91*7=637, c3=637*13=8281, c4=8281*7=57967, c5=57967*13=753571
      const expectedAlt = ["91", "637", "8281", "57967", "753571"].slice(
        0,
        nOutputs,
      );

      const { proof, publicSignals } = await snarkjs.plonk.fullProve(
        { a: "7", b: "13" },
        wasmPath,
        zkeyPath,
      );

      expect(publicSignals).toEqual(expectedAlt);

      const vk = JSON.parse(fs.readFileSync(vkPath, "utf8"));
      const valid = await snarkjs.plonk.verify(vk, publicSignals, proof);
      expect(valid).toBe(true);
    });

    it("rejects tampered public signal", async () => {
      const { proof, publicSignals } = await snarkjs.plonk.fullProve(
        { a: "3", b: "11" },
        wasmPath,
        zkeyPath,
      );

      // Tamper the first public signal
      publicSignals[0] = "999";

      const vk = JSON.parse(fs.readFileSync(vkPath, "utf8"));
      const valid = await snarkjs.plonk.verify(vk, publicSignals, proof);
      expect(valid).toBe(false);
    });
  },
);
