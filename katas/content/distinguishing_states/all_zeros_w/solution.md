We can see that each basis state in the W state always has exactly one qubit in the $\ket{1}$ state, while in the $\ket{0...0}$ state, all qubits are in the $\ket{0}$ state.

We can use this to arrive to the solution: we will count the number of qubits that were measured in `One` state; if this number equals 1, we had a W state, if it equals 0, we know it was the $\ket{0...0}$ state.

> Note the use of a mutable variable `countOnes` to store the number of qubits measured in `One` state, and the use of a ternary operator `condition ? trueValue | falseValue` to express the return value.

@[solution]({
    "id": "distinguishing_states__all_zeros_w_solution_A",
    "codePath": "SolutionA.qs"
})

`MeasureInteger()` can also be used in this task to make the solution shorter.

@[solution]({
    "id": "distinguishing_states__all_zeros_w_solution_B",
    "codePath": "SolutionB.qs"
})
