**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. An integer $k$ such that $0 \le k < N$.

**Goal:** 
Implement a marking oracle which calculates the $k$-th bit of the input array, that is, the function
$$f(x) = x_k$$

In other words, for each basis state $\ket{x}$, flip the state of the target qubit $\ket{y}$ if $f(x) = 1$, and leave it unchanged otherwise.
Leave the qubits in the input register in the same state they started in. 
Your solution should work on inputs in superposition, and not use any measurements.
