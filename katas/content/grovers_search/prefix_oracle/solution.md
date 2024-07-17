In this problem, the value of the function we're evaluating does not depend on the state of the qubits after qubit $P$. This means that we can just ignore them and consider only the qubits that matter - `x[... Length(p) - 1]`.

Once we do that, the problem becomes much simpler: flip the state of the target qubit if the input qubits are in the given state. That's the definition of a controlled gate with arbitrary controls, and we can apply that easily using the library operation `ApplyControlledOnBitString`.

@[solution]({
    "id": "grovers_search__prefix_oracle_solution",
    "codePath": "./Solution.qs"
})
