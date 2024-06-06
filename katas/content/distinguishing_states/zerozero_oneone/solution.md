Both qubits in the input array are in the same state: for $\ket{00}$ each individual qubit is in state $\ket{0}$, for $\ket{11}$ each individual qubit is in state $\ket{1}$. Therefore, if we measure one qubit, we will know the state of the other qubit.

In other words, if the first qubit measures as `One`, we know that the qubits in the input array are in state $\ket{11}$, and if it measures as `Zero`, we know they are in state $\ket{00}$.

> `condition ? truevalue | falsevalue` is Q#'s ternary operator: it returns `trueValue` if `condition` is true and `falseValue` otherwise.

@[solution]({
    "id": "distinguishing_states__zerozero_oneone_solution",
    "codePath": "Solution.qs"
})
