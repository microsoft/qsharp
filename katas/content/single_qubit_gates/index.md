# Single-Qubit Gates

@[section]({
    "id": "single_qubit_gates_overview",
    "title": "Overview"
})

This tutorial introduces you to single-qubit gates. Quantum gates are the quantum counterpart to classical logic gates, acting as the building blocks of quantum algorithms. Quantum gates transform qubit states in various ways, and can be applied sequentially to perform complex quantum calculations. Single-qubit gates, as their name implies, act on individual qubits. You can learn more at [Wikipedia](https://en.wikipedia.org/wiki/Quantum_logic_gate).

This tutorial covers the following topics:

- Matrix representation
- Ket-bra representation
- The most important single-qubit gates

@[exercise]({
    "id": "y_gate",
    "title": "The $Y$ gate",
    "descriptionPath": "./y_gate/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./y_gate/Verification.qs"
    ],
    "placeholderSourcePath": "./y_gate/Placeholder.qs",
    "solutionPath": "./y_gate/solution.md"
})

@[exercise]({
    "id": "global_phase_i",
    "title": "Applying a global phase $i$",
    "descriptionPath": "./global_phase_i/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./global_phase_i/Verification.qs"
    ],
    "placeholderSourcePath": "./global_phase_i/Placeholder.qs",
    "solutionPath": "./global_phase_i/solution.md"
})
