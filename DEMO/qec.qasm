// https://github.com/openqasm/openqasm/blob/main/examples/qec.qasm
// Repetition code syndrome measurement
include "stdgates.inc";

qubit[3] q;
qubit[2] a;
bit[3] c;
bit[2] syn;

def syndrome(qubit[3] d, qubit[2] a) -> bit[2] {
  bit[2] b;
  cx d[0], a[0];
  cx d[1], a[0];
  cx d[1], a[1];
  cx d[2], a[1];
  measure a[0] -> b[0];
  measure a[1] -> b[1];
  return b;
}

reset q[0];
reset q[1];
reset q[2];

reset a[0];
reset a[1];

x q[0]; // insert an error
barrier q;

syn = syndrome(q, a);

if(syn == 1) x q[0];
if(syn == 2) x q[2];
if(syn == 3) x q[1];

c[0] = measure q[0];
c[1] = measure q[1];
c[2] = measure q[2];
