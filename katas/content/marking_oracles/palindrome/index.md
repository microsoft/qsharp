**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which checks whether the input register is a palindrome, i.e., implements the function $f(x) = 1$ if $x$ is a palindrome, and 0 otherwise. A bit string is a palindrome if it equals its reverse, or, in other words, its first bit equals its last bit, its second bit equals its second-to-last bit, and so on.

For example, for $N = 3$ the input state $\ket{101}$ is a palindrome, and $\ket{001}$ is not.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.
