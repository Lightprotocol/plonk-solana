pragma circom 2.0.0;

// Minimal circuit: prover knows a, b such that a * b = c.
// All inputs are private by default. The output c becomes the public signal.
template Multiplier() {
    signal input a;
    signal input b;
    signal output c;

    c <== a * b;
}

component main = Multiplier();
