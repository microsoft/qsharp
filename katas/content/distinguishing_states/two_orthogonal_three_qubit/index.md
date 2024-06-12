**Input:** Three qubits (stored in an array of length 3) which are guaranteed to be in one of the two orthogonal states.

**Output:**
* 0 if they were in the state 
  $\ket{S_0} = \frac{1}{\sqrt{3}} \big(\ket{100} + \omega \ket{010} + \omega^2 \ket{001} \big)$,
* 1 if they were in the state 
  $\ket{S_1} = \frac{1}{\sqrt{3}} \big(\ket{100} + \omega^2 \ket{010} + \omega \ket{001} \big)$.

Here $\omega = e^{2i \pi/3}$.

The state of the qubits at the end of the operation does not matter.
