**Input:** Two qubits (stored in an array of length 2) which are guaranteed to be in one of the four orthogonal states.

**Output:**
* 0 if they were in the state 
  $\ket{S_0} = \frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11}\big)$,
* 1 if they were in the state 
  $\ket{S_1} = \frac{1}{2} \big(\ket{00} - \ket{01} + \ket{10} - \ket{11}\big)$,
* 2 if they were in the state 
  $\ket{S_2} = \frac{1}{2} \big(\ket{00} + \ket{01} - \ket{10} - \ket{11}\big)$,
* 3 if they were in the state 
  $\ket{S_3} = \frac{1}{2} \big(\ket{00} - \ket{01} - \ket{10} + \ket{11}\big)$.
  
The state of the qubits at the end of the operation does not matter.
