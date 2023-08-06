**Input:** Two qubits (stored in an array of length 2) which are guaranteed to be in one of the four basis states ($|00\rangle$, $|01\rangle$, $|10\rangle$, or $|11\rangle$).

**Output:**

* 0 if the qubits were in the $|00\rangle$ state,
* 1 if they were in the $|01\rangle$ state, 
* 2 if they were in the $|10\rangle$ state, 
* 3 if they were in the $|11\rangle$ state.

In this task and the subsequent ones the order of qubit states in task description matches the order of qubits in the array (i.e., $|10\rangle$ state corresponds to `qs[0]` in state $|1\rangle$ and `qs[1]` in state $|0\rangle$).

The state of the qubits at the end of the operation does not matter.
