OPENQASM 3.0;
include "stdgates.inc";
include "nonexistent.qasm";

// This file includes a missing file
qreg q[1];
h q[0];
