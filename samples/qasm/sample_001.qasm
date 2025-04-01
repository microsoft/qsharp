OPENQASM 3;
include "stdgates.inc";

gate hgate q { h q; }
gate xgate q { x q; }

const int[32] N = 4;

qubit[4] q;
qubit ancilla;

def deutsch_jozsa(qubit[N] q_func, qubit ancilla_q) {
  for int i in [0:N-1] { hgate q_func[i]; }
  hgate ancilla_q;
  for int i in [0:N-1] { cx q_func[i], ancilla_q; }
  for int i in [0:N-1] { hgate q_func[i]; }
}

deutsch_jozsa(q, ancilla);

output bit[4] result;
result[0] = measure q[0];
result[1] = measure q[1];
result[2] = measure q[2];
result[3] = measure q[3];