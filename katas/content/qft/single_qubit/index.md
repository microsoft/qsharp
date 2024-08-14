**Input**: A qubit in state $\ket{\psi} = x_0 \ket{0} + x_1 \ket{1}$.

**Goal**: 
Apply quantum Fourier transform to this qubit, that is, transform it to a state $\frac1{\sqrt2} \big((x_0 + x_1) \ket{0} + (x_0 - x_1) \ket{1}\big)$.

In other words, transform each basis state $\ket{j}$ into $\frac1{\sqrt2} (\ket{0} + e^{2\pi i \cdot \frac{j}{2}} \ket{1} )$.