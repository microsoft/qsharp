**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which checks whether the number represented by the bit string is divisible by $3$.
Use little endian notation to convert the bit string to an integer, i.e., the least significant bit is stored in `x[0]`.

For example, for $N = 3$ the basis states that should be marked are $\ket{000} = \ket{0}$, $\ket{110} = \ket{3}$, and $\ket{011} = \ket{6}$.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.