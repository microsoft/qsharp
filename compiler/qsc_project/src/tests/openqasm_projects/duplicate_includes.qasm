OPENQASM 3.0;
include "included.qasm";
include "included.qasm"; // Duplicate include

// File with duplicate includes
qreg q[1];
U(0, 0, 0) q[0];
