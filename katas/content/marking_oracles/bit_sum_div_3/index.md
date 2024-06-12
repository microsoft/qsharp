**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which checks whether the sum of bits in the bit string is divisible by $3$.

For example, for $N = 3$ the only basis states that should be marked are $\ket{000}$ and $\ket{111}$.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.