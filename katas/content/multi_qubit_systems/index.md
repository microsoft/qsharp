# Multi-Qubit Systems

This tutorial introduces you to multi-qubit systems - their representation in mathematical notation and in Q# code.

If you are not familiar with the [single-qubit systems](../Qubit/Qubit.ipynb), we recommend that you complete that tutorial first

This tutorial covers the following topics:

- Vector representation of multi-qubit systems
- Entangled and separable states
- Dirac notation

## Q# Example: Using multiple qubits

This example shows you how to allocate multiple qubits in Q# and examine their joint state. It uses single-qubit gates for manipulating the individual qubit states. It also uses the function `DumpMachine` to show the state of the quantum simulator.
When printing the state of multi-qubit systems, this function outputs the same information for each multi-qubit basis state.

@[example]({"id": "multiple_qubits", "codePath": "./MultipleQubits.qs"})

### Exercise: Prepare a basis state

**Input:** A two-qubit system in the basis state $|00\\rangle = \\begin{bmatrix} 1 \\\\ 0 \\\\ 0 \\\\ 0 \\end{bmatrix}$.

**Goal:** Transform the system into the basis state $|11\\rangle = \\begin{bmatrix} 0 \\\\ 0 \\\\ 0 \\\\ 1 \\end{bmatrix}$.

@[exercise]({
"id": "prepare_basis_state",
"codeDependencies": [],
"verificationSourcePath": "prepare_basis_state/Verification.qs",
"placeholderSourcePath": "prepare_basis_state/Placeholder.qs",
"solutionSourcePath": "prepare_basis_state/Solution.qs",
"solutionDescriptionPath": "prepare_basis_state/solution.md"
})

## Conclusion

As you've seen in the exercises, you can prepare separable multi-qubit states using only single-qubit gates.
