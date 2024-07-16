If we were not asked to maintain the state of the qubits, one approach would be to measure both the qubits separately in the computational basis, and check if the result is the same for both the measurements. If the measurement results are equal, the input state must have been a superposition of $\ket{00}$ and $\ket{11}$, while different measurement outcomes will imply that the input state must have been a superposition of $\ket{01}$ and $\ket{10}$. However, in these measurements we will lose the information about the original superposition states: a state $\alpha \ket{00} + \beta \ket{11}$ will collapse to either $\ket{00}$ or $\ket{11}$, and we won't be able to recover the information about the coefficients $\alpha$ and $\beta$.

We need to measure the *parity* of the state without collapsing it all the way to the basis states. Pauli measurements can be used for joint measurements involving multiple qubits. For this task we need to do the $Z \otimes Z$ measurement on both qubits.

A joint measurement using $Z \otimes Z$ operator can be thought of as projecting the measured state to one of the two eigenspaces of $Z \otimes Z$ with $+1$ and $-1$ as the corresponding eigenvalues. The measurement returns `Zero` if the measured state is projected to the space with an eigenvalue of $+1$, and a result of `One` if projected to the space with an eigenvalue of $-1$.

As we've seen in the tutorial, the state $\alpha \ket{00} + \beta \ket{11}$ is an eigenstate of the $Z \otimes Z$ operator with the eigenvalue $+1$, and the state $\alpha \ket{01} + \beta \ket{10}$ is an eigenstate with the eigenvalue $-1$.
Hence, we can use this joint measurement to recognize which of the superposition states we were given while preserving the initial superposition state.

In Q#, the operation `Measure` can be used to measure multiple qubits using an array of `Pauli` constants (`PauliI`, `PauliX`, `PauliY`, or `PauliZ`) that define the basis for measurement.

@[solution]({
    "id": "multi_qubit_measurements__joint_measurements_solution",
    "codePath": "Solution.qs"
})
