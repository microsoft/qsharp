**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $\ket{0 \dots 0}$ state.
2. A bit string of length $N$ represented as `Bool[]`. Bit values `false` and `true` correspond to $\ket{0}$ and $\ket{1}$ states. You're guaranteed that the first bit of the bit string is `true`.

**Goal:**  Change the state of the qubits to an equal superposition of $\ket{0 \dots 0}$ and the basis state given by the bit string.

> For example, for the bit string `[true, false]` the state required is $\frac{1}{\sqrt{2}}\big(\ket{00} + \ket{10}\big)$.
