This time, we need to do the $X \otimes X$ measurement on both qubits.

The state $\alpha \ket{++} + \beta \ket{--}$ is an eigenstate of the $X \otimes X$ operator with the eigenvalue $+1$, and the state $\alpha \ket{+-} + \beta \ket{-+}$ is an eigenstate with the eigenvalue $-1$.
Hence, we can use this joint measurement to recognize which of the superposition states we were given while preserving the initial superposition state.

@[solution]({
    "id": "qec_shor__xx_measurement_solution",
    "codePath": "Solution.qs"
})
