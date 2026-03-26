pragma circom 2.0.0;

template Mul1() {
    signal input a;
    signal input b;
    signal output c1;
    c1 <== a * b;
}

component main = Mul1();
