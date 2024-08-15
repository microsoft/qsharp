**Inputs:**

1. The number of vertices in the graph $V$ ($V \leq 6$).
2. An array of $E$ tuples of integers, representing the edges of the graph ($E \leq 12$).
Each tuple gives the indices of the start and the end vertices of the edge.
The vertices are numbered $0$ through $V - 1$.
3. An array of $2V$ qubits in an arbitrary state $\ket{x}$ representing the assigned coloring of the vertices using four colors ($nBits = 2$) in the same format as in the exercise "Read Coloring From a Qubit Array".
4. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
5. An index of a vertex in the graph between $0$ and $V - 1$, inclusive.

**Goal:**
Implement a quantum oracle which checks whether the given vertex is weakly colored in the given coloring of the given graph,
that is, whether it is either isolated or is connected to at least one vertex of a different color.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.