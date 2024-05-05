**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $|0 \dots 0\rangle$ state.
2. A bit string of length $N$ represented as `Bool[]`. Bit values `false` and `true` correspond to $|0\rangle$ and $|1\rangle$ states. You are guaranteed that the first bit of the bit string is `true`.

**Goal:**  Change the state of the qubits to an equal superposition of $|0 \dots 0\rangle$ and the basis state given by the bit string.

> For example, for the bit string `[true, false]` the state required is $\frac{1}{\sqrt{2}}\big(|00\rangle + |10\rangle\big)$.