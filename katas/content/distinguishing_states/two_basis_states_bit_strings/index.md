**Inputs:**

1. $N$ qubits (stored in an array of length $N$) which are guaranteed to be in one of the two basis states described by the given bit strings.
2. Two bit strings represented as `Bool[]`s.

**Output:**

* 0 if the qubits were in the basis state described by the first bit string,
* 1 if they were in the basis state described by the second bit string.

Bit values `false` and `true` correspond to $\ket{0}$ and $\ket{1}$ states. You are guaranteed that both bit strings have the same length as the qubit array, and that the bit strings differ in at least one bit.

**You can use exactly one measurement.** The state of the qubits at the end of the operation does not matter.

> Example: for bit strings `[false, true, false]` and `[false, false, true]` return 0 corresponds to state $\ket{010}$, and return 1 corresponds to state $\ket{001}$.
