**Inputs:**

1. `qs`: an array of $N$ qubits in the $\ket{0}$ states,
2. `bases`: a `Bool` array of length $N$; 
    `bases[i]` indicates the basis that should be used to prepare the qubit `i`:
    * `false`: use the basis $\ket{0}$ / $\ket{1}$ (computational),
    * `true`: use the basis $\ket{+}$ / $\ket{-}$ (Hadamard).
3. `bits`: a `Bool` array of length $N$;
    `bits[i]` indicates the bit to encode in the i-th qubit: `false` = 0, `true` = 1.

**Goal:**  Prepare the qubits in the described state.
