**Input:** An array of $N$ qubits in an arbitrary state $\ket{x}$.

**Goal:** Apply the phase oracle $U_f$ for $f(x) = \text{most significant bit of } x$ to the qubits.
That is, apply a relative phase $(-1)^{f(x)}$ to each basis state $\ket{x}$.
$x$ is encoded using big endian, with the most significant bit stored first.
