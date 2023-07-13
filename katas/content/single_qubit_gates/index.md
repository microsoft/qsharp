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

## Preparing a $|-\rangle$ state

**Input:** A qubit in state $|0\rangle$.

**Goal:** Transform the qubit into state $|-\rangle$.

@[exercise]({
    "id": "prepare_minus",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./prepare_minus/Verification.qs",
    "placeholderSourcePath": "./prepare_minus/Placeholder.qs",
    "solutionSourcePath": "./prepare_minus/Solution.qs",
    "solutionDescriptionPath": "./prepare_minus/solution.md"
})

## Three-fourths phase

**Input:** A qubit in an arbitrary state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$.

**Goal:** Use several phase shift gates to apply the transformation represented by the following matrix to the given qubit:

$$\begin{bmatrix} 1 & 0 \\ 0 & e^{3i\pi/4} \end{bmatrix}$$

@[exercise]({
    "id": "three_quarters_pi_phase",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./three_quarters_pi_phase/Verification.qs",
    "placeholderSourcePath": "./three_quarters_pi_phase/Placeholder.qs",
    "solutionSourcePath": "./three_quarters_pi_phase/Solution.qs",
    "solutionDescriptionPath": "./three_quarters_pi_phase/solution.md"
})

## Preparing a rotated state

**Inputs:**

1. Real numbers $\alpha$ and $\beta$ such that $\alpha^2 + \beta^2 = 1$.
2. A qubit in state $|0\rangle$.

**Goal:** Use a rotation gate to transform the qubit into state $\alpha|0\rangle -i\beta|1\rangle$.

> You will probably need functions from the [Math](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math) namespace, specifically [ArcTan2](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arctan2).
>
> You can assign variables in Q# by using the `let` keyword: `let num = 3;` or `let result = Function(input);`

@[exercise]({
    "id": "prepare_rotated_state",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./prepare_rotated_state/Verification.qs",
    "placeholderSourcePath": "./prepare_rotated_state/Placeholder.qs",
    "solutionSourcePath": "./prepare_rotated_state/Solution.qs",
    "solutionDescriptionPath": "./prepare_rotated_state/solution.md"
})
