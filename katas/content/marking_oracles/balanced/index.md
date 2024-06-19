**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which checks whether the input register is a balanced bit string, i.e., whether it contains exactly $N/2$ $0$'s and $N/2$ $1$'s. $N$ will be an even number.

For example, for $N = 4$ basis states $\ket{0011}$ and $\ket{0101}$ are balanced, and $\ket{0010}$ and $\ket{1111}$ are not.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.