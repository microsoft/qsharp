**Inputs:**

1. `qs`: an array of $N$ qubits;  
   each qubit is in one of the following states: $|0\rangle$, $|1\rangle$, $|+\rangle$, $|-\rangle$. 
2. `bases`: a `Bool` array of length $N$; 
   `bases[i]` indicates the basis that should be used to measure the qubit `i`:
    * `false`: use the basis $\ket{0}$ / $\ket{1}$ (computational),
    * `true`: use the basis $\ket{+}$ / $\ket{-}$ (Hadamard).

**Goal:**  Measure each qubit in the corresponding basis and return an array of results as Boolean values, encoding measurement result `Zero` as `false` and `One` as `true`. 
The state of the qubits at the end of the operation does not matter.
