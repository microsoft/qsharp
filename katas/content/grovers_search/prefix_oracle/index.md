**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. A bit pattern $p$ of length $P$ represented as a `Bool[]` ($1 ≤ P ≤ N$).

**Goal:** 
Implement a quantum oracle which checks whether the pattern $p$ is the prefix of the input register, that is, for all $j$ between $0$ and $P - 1$, inclusive, $p_j = x_j$. (`false` and `true` values represent states $\ket{0}$ and $\ket{1}$, respectively).

For example, for $N = 3$ two bit strings start with the prefix `[true, false]`: `[true, false, false]` and `[true, false, true]`.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.