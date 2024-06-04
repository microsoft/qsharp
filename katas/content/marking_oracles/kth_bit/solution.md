Since the effect of this oracle depends only on the value of the $k$-th qubit, we can ignore the rest of the qubits and focus on just `x[k]`. We need to flip the state of the target qubit if the input qubit `x[k]` is in the $\ket{1}$ state and leave it unchanged otherwise - this is exactly the effect of the $CNOT$ gate with the qubit `x[k]` used as control and the qubit `y` used as target.

@[solution]({
    "id": "marking_oracles__kth_bit_solution",
    "codePath": "./Solution.qs"
})
