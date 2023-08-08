The starting state can be represented as follows:
$$ \begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix} = |0\rangle \otimes |0\rangle $$

The goal state can be represented as follows:
$$ \begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix} = |1\rangle \otimes |1\rangle $$

Applying an **X** gate to a qubit in the $|0\rangle$ state transforms the qubit state into the $|1\rangle$ state. So, if we apply the **X** gate on the first qubit and the second qubit, we get the desired state.

@[solution]({
"id": "prepare_basis_state_solution",
"codePath": "solution.qs"
})
