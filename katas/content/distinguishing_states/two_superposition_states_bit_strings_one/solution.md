Like in the previous solution, we are looking for the index Q where the two bit strings differ. Let's define a function `FindFirstSuperpositionDiff()` which searches for an index Q which has two properties:

1. The value of all arrays in `bits1` at the index Q is either `true` or `false`, and the value of all arrays in `bits2` at the index Q is either `true` or `false`. If this is not the case, you cannot be sure that measuring the corresponding qubit will always return the same result.
   > For example, if you are given the state $\frac{1}{\sqrt2}\big(\ket{010} + \ket{011}\big)$, and if you measure the third qubit, $50\%$ of the time you will get $0$ and $50\%$ of the time $1$, therefore to get reliable information you want to measure one of the first two qubits.

2. This value is different for `bits1` and `bits2`.
   > Indeed, if you want to distinguish states $\frac{1}{\sqrt2}\big(\ket{010} + \ket{011}\big)$ and $\frac{1}{\sqrt2}\big(\ket{000} + \ket{001}\big)$, there are two qubits that will produce a fixed measurement result for each state - the first one and the second one. However, measuring the first qubit will give $0$ for both states, while measuring the second qubit will give $1$ for the first state and $0$ for the second state, allowing to distinguish the states.

To do this, we will iterate over all qubit indices, and for each of them we'll calculate the number of 1s in that position in `bits1` and `bits2`.

1. The first condition means that this count should equal 0 (if all bit strings have 0 bit in this position) or the length of the array of bit strings (if all bit strings have 1 bit in this position).
2. The second condition means that this count is different for `bits1` and `bits2`, i.e., one of the counts should equal 0 and another one - the length of the corresponding array of bit strings.

The second step is very similar to the previous exercise: given the index we just found, we measure the qubit on that position.
Here we use the library function `ResultAsBool(M(qs[diff]))` that returns `true` if the measurement result is `One` and `false` if the result is `Zero`; a call to this library function is equivalent to comparison `M(qs[diff]) == One` that was used in the previous task.

@[solution]({
    "id": "distinguishing_states__two_superposition_states_bit_strings_solution_one",
    "codePath": "Solution.qs"
})
