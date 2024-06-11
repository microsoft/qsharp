**Inputs:**

1. Angle $\alpha$, in radians, represented as a `Double`.
2. A qubit which is guaranteed to be either in $\ket{A}$ or the $\ket{B}$ state, where 
$$\ket{A} = \cos \alpha \ket{0} + \sin \alpha \ket{1}$$ 
$$\ket{B} = - \sin \alpha \ket{0} + \cos \alpha \ket{1}$$

**Output:** `true` if the qubit was in the $\ket{A}$ state, or `false` if it was in the $\ket{B}$ state. The state of the qubit at the end of the operation does not matter. 
