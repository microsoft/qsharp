**Inputs:**

1. An array of $nBits$ qubits in an arbitrary state $\ket{x_0}$ representing the first color.
2. An array of $nBits$ qubits in an arbitrary state $\ket{x_1}$ representing the second color.
3. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:**
Implement a quantum oracle which checks whether the two given colors are equal.
In other words, flip the state of qubit $\ket{y}$ if $x_0 = x_1$, and leave it unchanged otherwise.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.