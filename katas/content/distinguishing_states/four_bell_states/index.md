**Input:** Two qubits (stored in an array of length 2) which are guaranteed to be in one of the four Bell states.

**Output:**
* 0 if they were in the state $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big(\ket{00} + \ket{11}\big)$,
* 1 if they were in the state $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} \big(\ket{00} - \ket{11}\big)$,
* 2 if they were in the state $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} \big(\ket{01} + \ket{10}\big)$,
* 3 if they were in the state $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} \big(\ket{01} - \ket{10}\big)$.
  
The state of the qubits at the end of the operation does not matter.
