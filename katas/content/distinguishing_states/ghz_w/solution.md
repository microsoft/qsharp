As we've seen in the previous task, each of the basis states that form the W state will have exactly one qubit in the $\ket{1}$ state. Basis states that forms the GHZ state will either have all qubits in the $\ket{1}$ state or all qubits in the $\ket{0}$ state.

This means that if we count the number of qubits that were measured in the `One` state, we'll get 1 for the W state, and 0 or $N$ for the GHZ state. The code ends up almost the same as in the previous task (in fact, you can use this exact code to solve the previous task).

@[solution]({
    "id": "distinguishing_states__ghz_w_solution",
    "codePath": "Solution.qs"
})
