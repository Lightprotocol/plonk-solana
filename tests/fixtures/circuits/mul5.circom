pragma circom 2.0.0;

template Mul5() {
    signal input a;
    signal input b;
    signal output c1;
    signal output c2;
    signal output c3;
    signal output c4;
    signal output c5;
    c1 <== a * b;
    c2 <== c1 * a;
    c3 <== c2 * b;
    c4 <== c3 * a;
    c5 <== c4 * b;
}

component main = Mul5();
