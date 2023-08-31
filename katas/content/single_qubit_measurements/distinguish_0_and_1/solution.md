The input qubit is guaranteed to be either in basis state $|0\rangle$ or $|1\rangle$. This means that when measuring the qubit in the computational basis, the measurement will report the input state without any doubt.

In Q# the operation [`M`](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.m) can be used to measure a single qubit in the computational basis. The measurement result is a value of type `Result` - the operation `M` will return `One` if the input qubit was in the $|1\rangle$ state and `Zero` if the input qubit was in the $|0\rangle$ state. Since we need to encode the first case as `false` and the second one as `true`, we can return the result of equality comparison between measurement result and `Zero`.

@[solution]({
"id": "single_qubit_measurements__distinguish_0_and_1_solution",
"exerciseId": "distinguish_0_and_1",
"codePath": "solution.qs"
})
