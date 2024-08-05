A weak coloring is valid when each of the vertices is weakly colored, that is, has either no neighbors or has at least one neighbor of a different color. This means that the solution needs to iterate through all vertices and check this condition for each of them, tracking separately the number of vertices connected to it with an edge and the existence of a connected vertex of a different color.

@[solution]({
    "id": "solving_graph_coloring__weak_coloring_classical_solution",
    "codePath": "./Solution.qs"
})
