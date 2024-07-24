**Inputs:**

1. The number of vertices in the graph $V$ ($V \leq 6$).
2. An array of $E$ tuples of integers, representing the edges of the graph ($E \leq 12$).
Each tuple gives the indices of the start and the end vertices of the edge.
The vertices are numbered $0$ through $V - 1$.
3. An array of $2V$ qubits in an arbitrary state $\ket{x}$ representing the assigned coloring of the vertices using four colors ($nBits = 2$) in the same format as in the exercise "Read Coloring From a Qubit Array".
3. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

For example, the graph `0 -- 1 -- 2` has $V = 3$ and `edges = [(0, 1), (1, 2)]`. 
The color assignments for this graph are represented with $6$ qubits; some of the valid colorings are $\ket{000110}$ and $\ket{001100}$.

**Goal:**
Implement a quantum oracle which checks whether the given coloring of this graph is a valid vertex coloring,
that is, whether the colors assigned to each pair of vertices connected with an edge are different.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.