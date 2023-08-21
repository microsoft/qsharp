We know that applying the Hadamard gate $H$ on the computational basis states $|0\rangle$ and $|1\rangle$ results in Hadamard basis states $|+\rangle$ and $|-\rangle$, respectively.
We are given a qubit in the state $|0\rangle$. We first apply the Pauli $X$ gate to turn it into $X|0\rangle=|1\rangle$, and then apply the $H$ gate, turning the qubit into the required $H|1\rangle=|-\rangle$ state.

@[solution]({
    "id": "prepare_minus_solution",
    "codePath": "./Solution.qs"
})
