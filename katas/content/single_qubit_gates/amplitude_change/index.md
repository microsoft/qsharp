**Inputs:**

1. Angle $\alpha$, in radians, represented as Double.
2. A qubit in state $\ket{\psi} = \beta \ket{0} + \gamma \ket{1}$.

**Goal:** Change the state of the qubit as follows:

- If the qubit is in state $\ket{0}$, change its state to $\cos \alpha \ket{0} + \sin \alpha \ket{1}$.
- If the qubit is in state $\ket{1}$, change its state to $-\sin \alpha \ket{0} + \cos \alpha \ket{1}$.
- If the qubit is in superposition, change its state according to the effect on basis vectors.
