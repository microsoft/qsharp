**Input:** A qubit in an arbitrary state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$.

**Goal:** Apply the phase oracle $U_f$ for $f(x) = 1 - x$ to the qubit.
That is, apply a relative phase $(-1)^{f(x)}$ to each basis state $\ket{x}$.

<details>
<summary><strong>Need a hint?</strong></summary>
We can represent the effect of the oracle as

$$U_f |x\rangle = (-1)^{1-x} |x\rangle = (-1) \cdot (-1)^x |x\rangle$$

Can you get this effect by combining some of the previous oracles implementations?
</details>
