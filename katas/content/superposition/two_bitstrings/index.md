**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $|0 \dots 0\rangle$ state.
2. Two bit strings of length $N$ represented as `Bool[]`s. Bit values `false` and `true` correspond to $|0\rangle$ and $|1\rangle$ states. You are guaranteed that the two bit strings differ in at least one bit.

**Goal:**  Change the state of the qubits to an equal superposition of the basis states given by the bit strings.

> For example, for bit strings `[false, true, false]` and `[false, false, true]` the state required is $\frac{1}{\sqrt{2}}\big(|010\rangle + |001\rangle\big)$.