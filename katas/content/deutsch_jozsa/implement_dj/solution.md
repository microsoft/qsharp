Follow the algorithm as outlined in the previous section:

1. Allocate $N$ qubits - they start in the $\ket{0}$ state.
2. Apply the $H$ gate to each qubit. You can use `ApplyToEach` operation for this, or a `for` loop.
3. Apply the oracle. The syntax for applying the oracle is the same as for applying any other gate or operation.
4. Apply the $H$ gate to each qubit again.
5. Measure each of the qubits. If any of the measurement results is `One`, the function is balanced.
You can't return `false` as soon as you encounter a `One` result, though, since you need to return all qubits to $\ket{0}$ state first.
Instead, you update the mutable variable that stores your result and continue through the rest of the loop.

@[solution]({
    "id": "deutsch_jozsa__implement_dj_solution",
    "codePath": "./Solution.qs"
})
