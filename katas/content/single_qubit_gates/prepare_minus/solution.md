We know that applying the Hadamard gate $H$ on the computational basis states $\ket{0}$ and $\ket{1}$ results in Hadamard basis states $\ket{+}$ and $\ket{-}$, respectively.
We are given a qubit in the state $\ket{0}$. We first apply the Pauli $X$ gate to turn it into $X\ket{0}=\ket{1}$, and then apply the $H$ gate, turning the qubit into the required $H\ket{1}=\ket{-}$ state.

@[solution]({
    "id": "single_qubit_gates__prepare_minus_solution",
    "codePath": "./Solution.qs"
})
