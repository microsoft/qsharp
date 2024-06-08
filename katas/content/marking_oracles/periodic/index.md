**Inputs:** 

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:** 
Implement a quantum oracle which checks whether the input register is periodic with any period, i.e., whether there exists a value $P < N$ such that for all $j$ between $0$ and $N - P - 1$, inclusive, $x_j = x_{j+P}$.

For example, for $N = 3$ a bit string `[false, true, false]` is periodic with period 2, so the bit string is periodic.

Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.
