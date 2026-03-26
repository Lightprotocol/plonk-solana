pragma circom 2.0.0;

template Mul3() {
    signal input a;
    signal input b;
    signal output c1;
    signal output c2;
    signal output c3;
    c1 <== a * b;
    c2 <== c1 * a;
    c3 <== c2 * b;
}

component main = Mul3();
