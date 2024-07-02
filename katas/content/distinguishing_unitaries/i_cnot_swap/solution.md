In this problem we are allowed to use the given unitary twice, so we can split our decision-making process in two phases:

1. Apply the unitary to the $\ket{11}$ state; $CNOT_{12}$ will yield the $\ket{10}$ state, $CNOT_{21}$ &mdash; $\ket{01}$, and both $I \otimes I$ and $SWAP$ gates will leave the state unchanged.
2. Now to distinguish $I \otimes I$ from $SWAP$, we can use the $\ket{01}$ state: $I \otimes I$ gate will leave it unchanged, while $SWAP$ will yield $\ket{10}$.

Library operation `MeasureInteger` measures all qubits of the array, resets them to $\ket{0}$, and returns the measurement results, using little-endian to convert the bit array to an integer.

@[solution]({
    "id": "distinguishing_unitaries__i_cnot_swap_solution",
    "codePath": "Solution.qs"
})
