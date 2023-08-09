**Inputs:**

1. Angle $\alpha$, in radians, represented as a `Double`.
2. A qubit which is guaranteed to be in either the $|A\rangle$ or the $|B\rangle$ state, where $|A\rangle = \cos \alpha |0\rangle - i \sin \alpha |1\rangle$ and $|B\rangle = - i \sin \alpha |0\rangle + \cos \alpha |1\rangle$.

**Output:** `true` if the qubit was in the $|A\rangle$ state, or `false` if it was in the $|B\rangle$ state. The state of the qubit at the end of the operation does not matter.

<details>
    <summary><strong>Need a hint?</strong></summary>
    <p>An $R_x$ rotation can be used to go from the computational basis $\{ \ket 0, \ket 1 \}$ to the $\{ \ket{A}, \ket{B} \}$ basis and vice versa.</p>
</details>
