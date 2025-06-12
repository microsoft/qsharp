OPENQASM 3.0;
include "circular_b.qasm";

// Circular dependency A -> B -> A
qreg q[1];
U(0, 0, 0) q[0];
