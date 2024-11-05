To implement this check, you need to start by counting the vertices connected to the given vertex 
and allocating an array of auxiliary qubits, one for each neighboring vertex. These qubits will have their states flipped if the vertices connected by the corresponding edge have the same color. 
This can be done easily using with the operation `Oracle_ColorEquality` from an earlier task, similarly to how you did in when validating the vertex coloring of the graph. 

Now, for the coloring of the vertex to be valid, it needs to either have no neighbors, or to have at least one neighbor of a different color, so the array of auxiliary qubits should be in any state but $\ket{1...1}$. To implement this, you can flip the target qubit $\ket{y}$ unconditionally using the $X$ gate, and then flip it again only if qubits are in state $\ket{1...1}$.

Make sure to uncompute the changes to the auxiliary qubits after you evaluate the final result to leave them clean before their release.

@[solution]({
    "id": "solving_graph_coloring__weak_coloring_one_vertex_solution",
    "codePath": "./Solution.qs"
})
