The input qubit is guaranteed to be either in basis state $\ket{0}$ or $\ket{1}$. This means that when measuring the qubit in the computational basis, the measurement will report the input state without any doubt.

In Q# the operation `M` can be used to measure a single qubit in the computational basis. The measurement result is a value of type `Result` - the operation `M` will return `One` if the input qubit was in the $\ket{1}$ state and `Zero` if the input qubit was in the $\ket{0}$ state. Since we need to encode the first case as `true` and the second one as `false`, we can return the result of equality comparison between measurement result and `One`.

@[solution]({
    "id": "distinguishing_states__zero_one_solution",
    "codePath": "Solution.qs"
})
