**Inputs:**

1. Angle $\alpha$, in radians, represented as a `Double`.
1. A qubit in some unknown state.

**Output:** Implement a measurement in the $\{\ket A, \ket B\}$ basis. Same as in the previous exercise, $|A\rangle = \cos \alpha |0\rangle - i \sin \alpha |1\rangle$ and $|B\rangle = - i \sin \alpha |0\rangle + \cos \alpha |1\rangle$. Return `Zero` if the measurement outcome is $A$, and `One` if the outcome is $B$.
The state of the qubit after the measurement should correspond to the measurement result.

<details>
    <summary><strong>Need a hint?</strong></summary>
    <p>An $R_x$ rotation can be used to go from the computational basis $\{ \ket 0, \ket 1 \}$ to the $\{ \ket{A}, \ket{B} \}$ basis and vice versa.</p>
</details>
