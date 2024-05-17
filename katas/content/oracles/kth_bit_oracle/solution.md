Since the effect of this oracle depends only on the value of the $k$-th qubit, we can ignore the rest of the qubits and focus on just `x[k]`.
We need to flip the phase of the qubit if it is in the $\ket{1}$ state and leave it unchanged otherwise - this is exactly the effect of the $Z$ gate.

@[solution]({
    "id": "oracles__kth_bit_oracle_solution",
    "codePath": "Solution.qs"
})
