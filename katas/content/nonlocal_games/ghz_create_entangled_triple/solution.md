1. Apply an X gate to the first and the second qubits to get the $\ket{110}$ state.
2. Appy an H gate to the first and the second qubits to get the following state:
$$\frac12 \big( \ket{000} - \ket{010} - \ket{100} + \ket{110} \big)$$
3. Flip the sign of the last term using a controlled Z gate with the first qubit as control and the second qubit as target (or vice versa):
$$\frac12 \big( \ket{000} - \ket{010} - \ket{100} -{\color{blue}\ket{110}} \big)$$
4. Now we have the right signs for each term, and the first and the last terms match those of the state we're preparing, so we just need to adjust the two middle terms.
To do this, we can use [ControlledOnBitString](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) operation to flip the state of the last qubit if the first two qubits are in $\ket{01}$ or in $\ket{10}$ states, which gives us:
$$\frac{1}{2} \big(\ket{000} - {\color{blue}\ket{011}} - {\color{blue}\ket{101}} - \ket{110} \big)$$

@[solution]({
    "id": "nonlocal_games__ghz_create_entangled_triple_solution",
    "codePath": "Solution.qs"
})
