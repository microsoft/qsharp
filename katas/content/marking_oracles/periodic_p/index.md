**Inputs:** 

1. An array of $N$ qubits in an arbitrary state $\ket{x}$ (input register),
2. A qubit in an arbitrary state $\ket{y}$ (target qubit),
3. An integer $P < N$.

**Goal:** 
Implement a quantum oracle which checks whether the input register is periodic with period $P$, i.e., for all $j$ between $0$ and $N - P - 1$, inclusive, $x_j = x_{j+p}$.

For example, for $N$ = 3 a bit string [false, true, false] is periodic with period 2.
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.