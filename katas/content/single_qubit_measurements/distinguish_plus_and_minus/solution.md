The input qubit is guaranteed to be either in basis state $\ket{+}$ or $\ket{-}$. This means that when measuring the qubit in the Pauli X basis, the measurement will report the input state without any doubt. (Recall that these states are eigenstates for the Pauli X matrix).

In Q# the operation `Measure` can be used to measure a qubit in Pauli basis of the user's choice. The operation returns a value of type `Result` and is `Zero` if the measured state corresponds to the eigenvalue $+1$, and `One` if it corresponds to the eigenvalue $-1$ of the Pauli operator.

Since the states $\ket +$ and $\ket -$ correspond to the eigenvalues $+1$ and $-1$ of the Pauli X operator, we can return the result of equality comparison between the measurement result and `One`.
Note that since `Measure` operation generally works with multiple qubits to perform multi-qubit measurements, it takes array parameters. To do a single-qubit measurement, you need to pass two arrays of one element, `[PauliX]` and `[q]`, rather than individual values.

@[solution]({
    "id": "single_qubit_measurements__distinguish_plus_and_minus_solution",
    "codePath": "Solution.qs"
})
