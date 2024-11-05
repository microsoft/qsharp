**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $\ket{0 \dots 0}$ state.
2. Two bit strings of length $N$ represented as `Bool[]`s. Bit values `false` and `true` correspond to $\ket{0}$ and $\ket{1}$ states. You're guaranteed that the two bit strings differ in at least one bit.

**Goal:**  Change the state of the qubits to an equal superposition of the basis states given by the bit strings.

> For example, for bit strings `[false, true, false]` and `[false, false, true]` the state required is $\frac{1}{\sqrt{2}}\big(\ket{010} + \ket{001}\big)$.
