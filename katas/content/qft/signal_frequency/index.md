**Input:**
$n$ qubits in the state

$$\frac1{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i \cdot \frac{Fk}{2^{n}}} \ket{k}$$

**Output:** The frequency $F$ of the "signal" encoded in this state ($0\leq F\leq 2^n-1$).

> For example, for $n = 2$ and the state $\frac12\big(\ket{0} + i\ket{1} - \ket{2} - i\ket{3} \big)$ the output should be $F = 1$.

The state of the qubits at the end of the operation does not matter.