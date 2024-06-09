**Input:** Two qubits (stored in an array of length 2) which are guaranteed to be in one of the four basis states ($\ket{00}$, $\ket{01}$, $\ket{10}$, or $\ket{11}$).

**Output:**

* 0 if the qubits were in the $\ket{00}$ state,
* 1 if they were in the $\ket{01}$ state,
* 2 if they were in the $\ket{10}$ state,
* 3 if they were in the $\ket{11}$ state.

In this task and the subsequent ones the order of qubit states in task description matches the order of qubits in the array (i.e., $\ket{10}$ state corresponds to `qs[0]` in state $\ket{1}$ and `qs[1]` in state $\ket{0}$).

The state of the qubits at the end of the operation does not matter.
