The goal is to flip the qubit $\ket{y}$ if and only if each of the matching pairs of qubits in the arrays `x0` and `x1` are in the same state.

You can check whether two qubits are in the same state by computing their $\textrm{XOR}$: if their state was the same, their $\textrm{XOR}$ will be $0$. You can use the $\textrm{CNOT}$ gate to compute $\textrm{XOR}$ of two qubits in place, for example, using the qubit of the array `x0` as the control and the qubit of the array `x1` as the target.

Once you've used $nBits$ $\textrm{CNOT}$ gates to compute all pairwise $\textrm{XOR}$'s, you'll need to flip the target qubit $\ket{y}$ only if all  qubits in `x1` are in the $\ket{0}$ state. 
This can be done by using zero-controlled $X$ gate, that is, `ApplyControlledOnInt(0, X, x1, y)`.

Finally, you need to uncompute the bitwise $\textrm{XOR}$'s to ensure that the qubits in `x1` are returned to their original state.

@[solution]({
    "id": "solving_graph_coloring__color_equality_solution",
    "codePath": "./Solution.qs"
})
