## Measurements in the Pauli bases
So far, we have discussed measurements done in the computational basis, i.e., the $\{ \ket 0, \ket 1\}$ basis. 

It is also possible to implement measurements in other orthogonal bases, such as the [Pauli X basis](../SingleQubitGates/SingleQubitGates.ipynb#Pauli-Gates), which consists of the two vectors $\ket + = \frac1{\sqrt2} \big(\ket 0 +\ket 1\big)$, and $\ket - = \frac1{\sqrt2} \big(\ket 0 -\ket 1\big)$. Q# has a built-in operation [Measure](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure) for measurements in the Pauli bases. 

> The `Measure` operation can be used for measuring multiple qubits in a multi-qubit system; however, in this tutorial we only consider measurements for single-qubit systems.

The eigenvalues of a Pauli matrix are $\pm 1$, with one eigenvector corresponding to each eigenvalue. For any chosen Pauli basis, the `Measure` operation returns `Zero` if the measurement outcome corresponds to the eigenvalue $+1$, and returns `One` if the measurement outcome corresponds to the eigenvalue $-1$. As in the case of the computational basis measurements, the wave function of the qubit collapses to the corresponding state after the measurement is executed. 

The probabilities of the outcomes are defined using a similar rule: to measure a state $\ket \psi$ in a Pauli basis $\{ \ket {b_0}, \ket {b_1}\}$, we represent it as a linear combination of the basis vectors
$$\ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1}$$

The probabilities of outcomes $0$ and $1$ will be defined as $|c_0|^2$ and $|c_1|^2$, respectively.

> Computational basis measurement is often referred to as measurement in Pauli Z basis. Indeed, the eigenvectors of the Z gate are $\ket 0$ and $\ket 1$, with eigenvalues $+1$ and $-1$, respectively.
