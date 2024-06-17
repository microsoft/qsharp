**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. An array $a$ of $K$ distinct indices in the input register ($1 ≤ K ≤ N$).
4. A bit pattern $r$ of length $K$ represented as a `Bool[]`.

**Goal:** 
Implement a quantum oracle which checks whether the input register matches the given pattern, i.e., the bits at the given indices match the corresponding bits in the pattern ("false" and "true" values represent states $\ket{0}$ and $\ket{1}$, respectively).
The value of input register $x_{a_j}$ should match the pattern element $r_j$ for all $j$ between $0$ and $K - 1$, inclusive.

For example, for $N = 3$ a pattern `[false, true]` at indices `[0, 2]` would match two basis states: $\ket{001}$ and $\ket{011}$.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.