OPENQASM 3;
include "stdgates.inc";
qubit q;
qubit[2] q2;
bit c;
bit[2] c2;
c2 = measure q2;
c = measure q;
