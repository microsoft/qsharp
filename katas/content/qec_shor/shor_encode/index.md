**Input**: nine qubits in the state $\ket{\psi} \otimes \ket{0 \ldots 0}$, where $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$ is the state of the first qubit, that is, `qs[0]`.

**Goal**: prepare a state $\alpha \ket{0_L} + \beta \ket{1_L}$ on these qubits, where
$$\ket{0_L} = \frac1{2\sqrt2} (\ket{000} + \ket{111}) \otimes (\ket{000} + \ket{111}) \otimes (\ket{000} + \ket{111})$$
and 
$$\ket{1_L} = \frac1{2\sqrt2} (\ket{000} - \ket{111}) \otimes (\ket{000} - \ket{111}) \otimes (\ket{000} - \ket{111})$$
