In this problem, the value of the function we're evaluating does not depend on the state of the qubits before qubit $P$ or after qubit $P + K - 1$. This means that we can just ignore them and consider only the qubits that matter - `x[p .. p + Length(r) - 1]`.

Once we do that, the problem becomes much simpler: flip the state of the target qubit if the input qubits are in the given state. That's the definition of a controlled gate with arbitrary controls, and we can apply that easily using the library operation `ApplyControlledOnBitString`.

@[solution]({
    "id": "marking_oracles__contains_substring_p_solution",
    "codePath": "./Solution.qs"
})
