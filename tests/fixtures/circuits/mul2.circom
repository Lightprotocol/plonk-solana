pragma circom 2.0.0;

template Mul2() {
    signal input a;
    signal input b;
    signal output c1;
    signal output c2;
    c1 <== a * b;
    c2 <== c1 * a;
}

component main = Mul2();
