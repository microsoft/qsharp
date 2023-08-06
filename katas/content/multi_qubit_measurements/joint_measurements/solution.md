If we were not asked to maintain the state of the qubits, one approach would be to measure both the qubits separately in the computational basis, and check if the result is the same for both the measurements. If the measurement results are equal, the input state must have been a superposition of $\ket{00}$ and $\ket{11}$, while different measurement outcomes will imply that the input state must have been a superposition of $\ket{01}$ and $\ket{10}$. However, in these measurements we will lose the information about the original superposition states: a state $\alpha |00\rangle + \beta |11\rangle$ will collapse to either $|00\rangle$ or $|11\rangle$, and we won't be able to recover the information about the coefficients $\alpha$ and $\beta$.

We need to measure the *parity* of the state without collapsing it all the way to the basis states. [Pauli measurements](https://docs.microsoft.com/en-us/quantum/concepts/pauli-measurements) can be used for joint measurements involving multiple qubits. For this task we apply the $Z \otimes Z$ measurement on both qubits.

A joint measurement using $Z \otimes Z$ operator can be thought as projecting the measured state to one of the two eigenspaces of $Z \otimes Z$ with $+1$ and $-1$ as the corresponding eigenvalues. The measurement returns `Zero` if the measured state is projected to the space with an eigenvalue of $+1$, and a result of `One` if projected to the space with an eigenvalue of $-1$.

As we've seen in the tutorial, the state $\alpha |00\rangle + \beta |11\rangle$ is an eigenstate of the $Z \otimes Z$ operator with the eigenvalue $+1$, and the state $\alpha |01\rangle + \beta |10\rangle$ is an eigenstate with the eigenvalue $-1$.
Hence, we can use this joint measurement to recognize which of the superposition states we were given while preserving the initial superposition state.

In Q#, the operation [`Measure`](https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure) can be used to measure multiple qubits using an array of [Pauli](https://docs.microsoft.com/en-us/quantum/user-guide/language/types?#primitive-types) constants that define the basis for measurement.

@[solution]({
"id": "joint_measurements_solution",
"codePath": "solution.qs"
})
