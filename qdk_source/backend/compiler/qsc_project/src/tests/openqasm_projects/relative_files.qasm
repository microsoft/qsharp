OPENQASM 3.0;
include "nested/gates.inc";

// Simple QASM file for testing
qreg q[2];
creg c[2];

gate_a q[0];
gate_b q[1];
measure q -> c;
