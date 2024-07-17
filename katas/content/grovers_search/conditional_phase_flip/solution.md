This operation is equivalent to the operation of flipping the sign of only the $\ket{0...0}$ basis state, with an additional global phase $-1$.
This means that you can implement it in two steps:

1. Flip the sign of the $\ket{0...0}$ basis state.
   You can do this by applying $X$ gates to all qubits, then using the Controlled $Z$ gate to flip the sign of $\ket{1...1}$, and then applying $X$ gates to all qubits again.
2. Apply a global phase $-1$ to the whole state using an `R1` gate.

@[solution]({
    "id": "grovers_search__conditional_phase_flip_solution",
    "codePath": "Solution.qs"
})
