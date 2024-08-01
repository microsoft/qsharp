**Inputs**:

1. A qubit in state $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$.
2. An array of $n$ qubits $j$ in state $\ket{j_1 j_2 ... j_n}$.

**Goal**: 
Change the state of the input qubits from $(\alpha \ket{0} + \beta \ket{1}) \otimes \ket{j_1 j_2 ... j_n}$ to $(\alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0.j_1 j_2 ... j_n} \ket{1}) \otimes \ket{j_1 j_2 ... j_n}$, where $0.j_1 j_2 ... j_n$ is a binary fraction corresponding to the basis state $j_1 j_2 ... j_n$ of the qubit array $j$.

> The qubit array can be in superposition as well;
the behavior of the transformation in this case is defined by its behavior on the basis states and the linearity of unitary transformations.
