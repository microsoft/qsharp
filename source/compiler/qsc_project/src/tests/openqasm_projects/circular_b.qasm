OPENQASM 3.0;
include "circular_a.qasm";

// Circular dependency B -> A -> B
qreg q[1];
U(0, 0, 0) q[0];
