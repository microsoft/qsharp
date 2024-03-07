The algorithm implementation is very similar to that of Deutsch-Jozsa algorithm, except the measurements step.

1. Allocate $N$ qubits.
2. Apply the $H$ gate to each qubit.
3. Apply the oracle.
4. Apply the $H$ gate to each qubit again.
5. Measure each of the qubits and convert measurement results to bits $0$ and $1$.

@[solution]({
    "id": "deutsch_jozsa__implement_bv_solution",
    "codePath": "./Solution.qs"
})
