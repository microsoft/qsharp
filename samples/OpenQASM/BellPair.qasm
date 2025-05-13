OPENQASM 3;
include "stdgates.inc";

qubit q1;
qubit q2;
bit r1;
bit r2;

h q1;
cx q1, q2;

r1 = measure q1;
r2 = measure q2;
