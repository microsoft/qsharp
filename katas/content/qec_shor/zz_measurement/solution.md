You need to measure the *parity* of the state without collapsing it all the way to the basis states. This means that you need to do the $Z \otimes Z$ measurement on both qubits.

A joint measurement using $Z \otimes Z$ operator can be thought of as projecting the measured state to one of the two eigenspaces of $Z \otimes Z$ with $+1$ and $-1$ as the corresponding eigenvalues. The measurement returns `Zero` if the measured state is projected to the space with an eigenvalue of $+1$, and a result of `One` if projected to the space with an eigenvalue of $-1$.

You can see that the state $\alpha \ket{00} + \beta \ket{11}$ is an eigenstate of the $Z \otimes Z$ operator with the eigenvalue $+1$, and the state $\alpha \ket{01} + \beta \ket{10}$ is an eigenstate with the eigenvalue $-1$.
Hence, you can use this joint measurement to recognize which of the superposition states you were given while preserving the initial superposition state.

@[solution]({
    "id": "qec_shor__zz_measurement_solution",
    "codePath": "Solution.qs"
})
