# %%

from qsharp.interop.qiskit import QSharpBackend
from qsharp import TargetProfile

my_qasm = """
OPENQASM 3.0;
include "stdgates.inc";
bit[2] c;
qubit[2] q;
reset q[0];
rx(3.141592653589793) q[0];
rx(3.141592653589793) q[1];
c[0] = measure q[0];
c[1] = measure q[1];
"""

backend = QSharpBackend()
qir = backend._qasm3_to_qir(my_qasm, search_path=".", target_profile=TargetProfile.Base)
print("*** BASE PROFILE QASM\n")
print("Input QASM")
print(my_qasm)
print("\nOutput QIR\n")
print(qir)

adaptive_qasm = """
OPENQASM 3.0;
include "stdgates.inc";

bit[1] c;
qubit[2] q;

h q[0];
cx q[0], q[1];
c[0] = measure q[1];

if (c[0] == true) {
   x q[1];
} else {
   sdg q[0];
}
"""

qir = backend._qasm3_to_qir(
    adaptive_qasm, search_path=".", target_profile=TargetProfile.Adaptive_RI
)
print("\n\n*** ADAPTIVE PROFILE QASM\n")
print("Input QASM")
print(adaptive_qasm)
print("\nOutput QIR\n")
print(qir)

# %%
