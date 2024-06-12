**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. A bit pattern $r$ of length $K$ represented as a `Bool[]` ($1 ≤ K ≤ N$).

**Goal:** 
Implement a quantum oracle which checks whether the input register contains the given pattern, i.e., whether there exists a position $P$ such that for all $j$ between $0$ and $K - 1$, inclusive, $r_j = x_{j+P}$.

For example, for $N = 3$ a bit string `[false, true, false]` contains a pattern `[true, false]` (starting at index 1).

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.