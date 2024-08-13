The starting state can be represented as follows:
$$ \begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix} = \ket{0} \otimes \ket{0} $$

The goal state can be represented as follows:
$$ \begin{bmatrix} 0 \\ 0 \\ 0 \\ 1 \end{bmatrix} = \ket{1} \otimes \ket{1} $$

Applying an $X$ gate to a qubit in the $\ket{0}$ state transforms the qubit state into the $\ket{1}$ state. So, if you apply the $X$ gate on the first qubit and the second qubit, you get the desired state.

@[solution]({
"id": "multi_qubit_systems__prepare_basis_state_solution",
"codePath": "Solution.qs"
})
