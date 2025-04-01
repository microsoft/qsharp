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
  for int i in [0:1] {
    measure a[i] -> b[i];
  }
  return b;
}

for int i in [0:2] {
  reset q[i];
}
for int i in [0:1] {
  reset a[i];
}
x q[0]; // insert an error
barrier q;
syn = syndrome(q, a);
if(syn == 1) x q[0];
if(syn == 2) x q[2];
if(syn == 3) x q[1];
for int i in [0:2]{
  c[i] = measure q[i];
}