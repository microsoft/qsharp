# Multi-Qubit Gates

This tutorial continues the introduction to quantum gates, focusing on applying quantum gates to multi-qubit systems.

This tutorial covers the following topics:

- Applying quantum gates to a part of the system
- $\\text{CNOT}$ and $\\text{SWAP}$ gates
- Controlled gates

## Preparing a Bell state

**Input:** Two qubits in state $|00\\rangle$, stored in an array of length 2.

**Goal:** Transform the system into the Bell state $\\Phi^+ = \\frac{1}{\\sqrt{2}}\\big(|00\\rangle + |11\\rangle\\big)$.

@[exercise]({
    "id": "preparing_bell_state",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./preparing_bell_state/Verification.qs",
    "placeholderSourcePath": "./preparing_bell_state/Placeholder.qs",
    "solutionSourcePath": "./preparing_bell_state/Solution.qs",
    "solutionDescriptionPath": "./preparing_bell_state/solution.md"
})
