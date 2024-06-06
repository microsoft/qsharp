**Inputs:** 

1. An array of $N$ qubits in an arbitrary state $\ket{x}$ (input register),
2. A qubit in an arbitrary state $\ket{y}$ (target qubit),
3. An array of $K$ distinct indices in the input register,
4. A bit pattern of length K represented as a Bool[] ($1 ≤ K ≤ N$).

**Goal:** 
Implement a quantum oracle which checks whether the input register matches the given pattern, i.e., the bits at the given indices match the corresponding bits in the pattern ("false" and "true" values represent states $\ket{0}$ and $\ket{1}$, respectively).

For example, for $N$ = 3 a pattern [false, true] at indices [0, 2] would match two basis states: $\ket{001}$ and $\ket{011}$.
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.