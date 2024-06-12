**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register). $N$ will be one of the values $3, 5, 7$.
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which calculates the majority function, i.e., $f(x) = 1$ if most of the bits in the bit string are $1$'s, and $0$ otherwise.
    
For example, for $N = 3$ majority function for basis states $\ket{001}$ and $\ket{000}$ is $0$, and for $\ket{101}$ and $\ket{111}$ is $1$.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.