To implement this oracle, you need to check whether each vertex is colored correctly and combine the results of individual checks. You'll need to allocate an array of qubits, one per vertex, that will be flipped if the corresponding vertex is weakly colored.
This can be done easily using the `Oracle_WeakColoring_OneVertex` operation defined in the previous task. Then, you need to check if all qubits in the array is in state $\ket{1...1}$ using the Controlled $X$ gate; if they are, the coloring is valid.

As usual, remember to uncompute the changes to the auxiliary qubits after you evaluate the final result to leave them clean before their release.

@[solution]({
    "id": "solving_graph_coloring__weak_coloring_quantum_solution",
    "codePath": "./Solution.qs"
})
