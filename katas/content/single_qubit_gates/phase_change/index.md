**Inputs:**

1. Angle α, in radians, represented as Double.
2. A qubit in state $|\psi\rangle = \beta |0\rangle + \gamma |1\rangle$.

**Goal:** Change the state of the qubit as follows:

- If the qubit is in state $|0\rangle$, don't change its state.
- If the qubit is in state $|1\rangle$, change its state to $e^{i\alpha} |1\rangle$.
- If the qubit is in superposition, change its state according to the effect on basis vectors: $\beta |0\rangle + {\color{red}{e^{i\alpha}}} \gamma |1\rangle$.
