**Inputs**:

1. A qubit in state $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$.
2. An array of $n$ bits $[j_1, j_2, ..., j_n]$, stored as `Bool[]`.

**Goal**: 
Change the state of the qubit to $\alpha \ket{0} + \beta \cdot \cdot e^{2\pi i \cdot 0.j_1 j_2 ... j_n} \ket{1}$, where $0.j_1 j_2 ... j_n$ is a binary fraction in big endian notation, similar to decimal fractions:

$$0.j_1 j_2 ... j_n = j_1 \cdot \frac{1}{2^1} + j_2 \cdot \frac{1}{2^2} + ... j_n \cdot \frac{1}{2^n}$$
