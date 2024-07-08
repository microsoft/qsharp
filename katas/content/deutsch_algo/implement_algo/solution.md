Follow the algorithm as outlined in the previous section:

1. Allocate a qubit - it starts in the $\ket{0}$ state.
2. Apply the $H$ gate to the qubit.
3. Apply the oracle. The syntax for applying the oracle is the same as for applying any other gate or operation.
4. Apply the $H$ gate to the qubit again.
5. Measure the qubit: if the measurement result is `Zero`, the function is constant, otherwise it's variable.
Since you need to return `true` if the function is constant, you can just return the result of comparing the measurement result with `Zero`.

@[solution]({
    "id": "deutsch_algo__implement_algo_solution",
    "codePath": "./Solution.qs"
})
