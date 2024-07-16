**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. A bit vector $r$ of length $N$ represented as a `Bool[]`.

**Goal:** 
Implement a marking oracle which calculates the scalar product function
$$f(x) = \bigoplus\limits_{i=0}^{N-1} r_i x_i$$

In other words, for each basis state $\ket{x}$, flip the state of the target qubit $\ket{y}$ if $f(x) = 1$, and leave it unchanged otherwise.
Leave the qubits in the input register in the same state they started in. 
Your solution should work on inputs in superposition, and not use any measurements.
