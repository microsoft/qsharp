You need to allocate an array of auxiliary qubits, one for each edge of the graph, that will have their states flipped if the vertices connected by the corresponding edge have the same color. 
This can be done easily using with the operation `Oracle_ColorEquality` from the previous task. 

Then, you need to check if these auxiliary qubits are still in state $\ket{0...0}$; if they are, all the necessary pairs of colors are distinct, the coloring is valid.

Since the coloring is provided as an array of qubits, with two qubits per vertex (2 qubits = 4 basis states = 4 colors), you have to take the correct chunks of the coloring to extract the color of each vertex. You can deduce that the coloring of vertex $j$ is encoded in qubits in positions $2j$ and $2j+1$.

Make sure to uncompute the changes to the auxiliary qubits after you evaluate the final result to leave them clean before their release.

@[solution]({
    "id": "solving_graph_coloring__vertex_coloring_quantum_solution",
    "codePath": "./Solution.qs"
})
