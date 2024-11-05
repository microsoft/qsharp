**Inputs:**

1. $n$ qubits in the $\ket{0 \dots 0}$ state.
2. An integer frequency $F$ ($0 \leq F \leq 2^{n}-1$).

**Goal:**  Change the state of the qubits to a periodic state with frequency $F$:

$$\frac1{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i \cdot \frac{Fk}{2^{n}}} \ket{k}$$

> For example, for $n = 2$ and $F = 1$ the goal state is $\frac12\big(\ket{0} + i\ket{1} - \ket{2} - i\ket{3} \big)$.
