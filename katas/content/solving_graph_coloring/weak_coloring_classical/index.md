**Inputs:**

1. The number of vertices in the graph $V$ ($V \leq 6$).
2. An array of $E$ tuples of integers, representing the edges of the graph ($E \leq 12$).
Each tuple gives the indices of the start and the end vertices of the edge.
The vertices are numbered $0$ through $V - 1$.
3. An array of $V$ integers, representing the vertex coloring of the graph. 
$i$-th element of the array is the color of the vertex number $i$.

For example, consider the triangular graph `0 -- 1 -- 2 -- 0` with $V = 3$ and `edges = [(0, 1), (1, 2), (0, 2)]`. 
The coloring `[0, 1, 0]` is a valid weak coloring for it despite the fact that the connected vertices $0$ and $2$ are assigned the same color, since each of them is also connected to the vertex $1$ of a different color.

**Output:**
True if the given weak coloring is valid (that is, each vertex either has no neighbors, or has at least one neighbor of a different color), and false otherwise.