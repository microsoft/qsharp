OPENQASM 3.0;
include "stdgates.inc";

// Simple QASM file for testing
qreg q[2];
creg c[2];

h q[0];
cx q[0], q[1];
measure q -> c;
