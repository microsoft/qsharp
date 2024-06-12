To solve this task, we will use two steps. Like many other programming languages, Q# allows you to write functions to make code more readable and reusable.

The first step is to find first bit that differs between bit strings `bit1` and `bit2`. For that we define a function `FindFirstDiff()` which loops through both `Bool[]`s and returns the first index where the bit strings differ.

The second step is implementing the main operation: once we have found the first different bit, we measure the qubit in the corresponding position to see whether it is in state $\ket{0}$ or $\ket{1}$. If it is in state $\ket{0}$, `res` takes the value `false`, if it is in state $\ket{1}$ it takes the value `true`.

`res == bits1[firstDiff]` compares the measurement result with the bit of `bits1` in the differing position. This effectively checks if the qubits are in the basis state described by the first or by the second bit string. The two possible outcomes are:

1. The qubits are in the state described by the first bit string; then `res` will be equal to `bits1[firstDiff]` and the method will return `0`.
2. The qubits are in the state described by the second bit string; then `res` will be not equal to `bits1[firstDiff]` (we know it has to be equal to `bits2[firstDiff]` which does not equal `bits1[firstDiff]`), and the method will return `1`.

@[solution]({
    "id": "distinguishing_states__two_basis_states_bit_strings_solution",
    "codePath": "Solution.qs"
})
