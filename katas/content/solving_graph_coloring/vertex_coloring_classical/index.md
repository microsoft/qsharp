**Inputs:**

1. The number of vertices in the graph $V$ ($V \leq 6$).
2. An array of $E$ tuples of integers, representing the edges of the graph ($E \leq 12$).
Each tuple gives the indices of the start and the end vertices of the edge.
The vertices are numbered $0$ through $V - 1$.
3. An array of $V$ integers, representing the vertex coloring of the graph. 
$i$-th element of the array is the color of the vertex number $i$.

For example, the graph `0 -- 1 -- 2` has $V = 3$ and `edges = [(0, 1), (1, 2)]`. 
Some of the valid colorings for it would be `[0, 1, 0]` and `[-1, 5, 18]`.
Notice that in this exercise, unlike in some of the later ones, the coloring is not limited to numbers from $0$ to $3$.

**Output:**
True if the given vertex coloring is valid (that is, no two vertices connected by an edge have the same color), and false otherwise.