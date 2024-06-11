**Input:** 
$N$ qubits (stored in an array of length $N$) which are guaranteed to be either in the $\ket{0...0}$ state or in the W state - an equal superposition of all $N$ basis states that have exactly one $\ket{1}$ in them. (For example, for $N = 3$ the W state is $\frac1{\sqrt3} (\ket{100} + \ket{010} + \ket{001})$).

**Output:**
* 0 if the qubits were in the $\ket{0...0}$ state,
* 1 if they were in the W state.

The state of the qubits at the end of the operation does not matter.
