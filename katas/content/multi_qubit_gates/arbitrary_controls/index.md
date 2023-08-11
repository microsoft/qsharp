**Input:**

1. `controls` - a register of $N$ qubits in an arbitrary state $|\phi\rangle$.
2. `target` - a qubit in an arbitrary state $|\psi\rangle$.
3. `controlBits` - an array of $N$ booleans, specifying what state each control qubit should be in order to apply the gate.

**Goal:** Apply the controlled `X` gate with the `controls` as control qubits and `target` as target, with the state specified by `controlBits` as controls. If the element of the array is `true`, the corresponding qubit is a regular control (should be in state $|1\rangle$), and if it is `false`, the corresponding qubit is an anti-control (should be in state $|0\rangle$).

> For example, if `controlBits = [true, false, true]`, the controlled `X` gate should only be applied if the control qubits are in state $|101\rangle$.

<details>
    <summary><strong>Need a hint?</strong></summary>
    <p>Consider using a library operation for this task. If you want to do it without a library operations, don't forget to reset the qubits back to the state they were originally in.</p>
</details>
