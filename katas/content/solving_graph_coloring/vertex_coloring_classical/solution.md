A graph coloring valid when the nodes connected by every edge have a different color. This means that we have to check every edge, see if the nodes have the same color, and if it is the case, return that the graph coloring is invalid. If every edge passed the test, we can safely say the graph coloring is valid.

Since the color of vertex $n$  is the $n$-th element of the `colors` array, we simply loop through every edge, which is a pair of vertex indices, and compare their colors.

@[solution]({
    "id": "solving_graph_coloring__vertex_coloring_classical_solution",
    "codePath": "./Solution.qs"
})
