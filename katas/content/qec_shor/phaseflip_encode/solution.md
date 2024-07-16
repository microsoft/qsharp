You can implement this encoding in two steps:

1. Use bit flip encoding to convert $\alpha \ket{000} + \beta \ket{100}$ into $\alpha \ket{000} + \beta \ket{111}$.
2. Apply a Hadamard gate to each qubit to convert each of the $\ket{0}$ states into $\ket{+}$ and each of the $\ket{1}$ states into $\ket{-}$.

@[solution]({
    "id": "qec_shor__phaseflip_encode_solution",
    "codePath": "Solution.qs"
})
