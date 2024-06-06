**Inputs:** 

1. An array of $N$ qubits in an arbitrary state $\ket{x}$ (input register),
2. A qubit in an arbitrary state $\ket{y}$ (target qubit),
3. A bit pattern of length $K$ represented as a Bool[] ($1 ≤ K ≤ N$),
4. An integer $0 ≤ P < N - K$.

**Goal:** 
Implement a quantum oracle which checks whether the input register contains the given pattern starting at the given position, i.e., for all $j$ between $0$ and $K - 1$, inclusive, $pattern_j = x_{j+p}$. ("false" and "true" values represent states $\ket{0}$ and $\ket{1}$, respectively).

For example, for $N = 3$ a bit string [false, true, false] contains a pattern [true, false] starting at index 1.
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.