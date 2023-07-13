# Single-Qubit Gates

This tutorial introduces you to single-qubit gates. Quantum gates are the quantum counterpart to classical logic gates, acting as the building blocks of quantum algorithms. Quantum gates transform qubit states in various ways, and can be applied sequentially to perform complex quantum calculations. Single-qubit gates, as their name implies, act on individual qubits. You can learn more at [Wikipedia](https://en.wikipedia.org/wiki/Quantum_logic_gate).

This tutorial covers the following topics:

- Matrix representation
- Ket-bra representation
- The most important single-qubit gates

## The $Y$ gate

**Input:** A qubit in an arbitrary state $|\\psi\\rangle = \\alpha|0\\rangle + \\beta|1\\rangle$.

**Goal:** Apply the Y gate to the qubit, i.e., transform the given state into $i\\alpha|1\\rangle - i\\beta|0\\rangle$.

@[exercise]({
    "id": "y_gate",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./y_gate/Verification.qs",
    "placeholderSourcePath": "./y_gate/Placeholder.qs",
    "solutionSourcePath": "./y_gate/Solution.qs",
    "solutionDescriptionPath": "./y_gate/solution.md"
})

## Applying a global phase $i$

**Input:** A qubit in an arbitrary state $|\\psi\\rangle = \\alpha|0\\rangle + \\beta|1\\rangle$.

**Goal:** Use several Pauli gates to change the qubit state to $i|\\psi\\rangle = i\\alpha|0\\rangle + i\\beta|1\\rangle$.

@[exercise]({
    "id": "global_phase_i",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./global_phase_i/Verification.qs",
    "placeholderSourcePath": "./global_phase_i/Placeholder.qs",
    "solutionSourcePath": "./global_phase_i/Solution.qs",
    "solutionDescriptionPath": "./global_phase_i/solution.md"
})

## Applying a $-1$ phase to $|0\rangle$ state

**Input:** A qubit in an arbitrary state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$.

**Goal:** Use several Pauli gates to change the qubit state to $- \alpha|0\rangle + \beta|1\rangle$, i.e., apply the transformation represented by the following matrix::

$$\begin{bmatrix} -1 & 0 \\ 0 & 1 \end{bmatrix}$$

@[exercise]({
    "id": "sign_flip_on_zero",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./sign_flip_on_zero/Verification.qs",
    "placeholderSourcePath": "./sign_flip_on_zero/Placeholder.qs",
    "solutionSourcePath": "./sign_flip_on_zero/Solution.qs",
    "solutionDescriptionPath": "./sign_flip_on_zero/solution.md"
})
