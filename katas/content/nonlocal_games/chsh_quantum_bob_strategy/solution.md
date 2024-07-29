A suitable $R_y$ rotation can be used to go from the computational basis $\{ \ket{0}, \ket{1} \}$ to the $\{ \ket{\psi_+}, \ket{\psi_-} \}$ basis and vice versa.

To implement the described transformation in Q#, we need to rotate the qubit by $\frac{\pi}{8}$ clockwise if `bit == false` or counterclockwise if `bit == true` and then perform a measurement.
We can do the rotation using the $R_y$ gate (note the negation of the Boolean parameter we need to do).

(See the lesson below for details on why Bob should follow this strategy.)

@[solution]({
    "id": "nonlocal_games__chsh_quantum_bob_strategy_solution",
    "codePath": "Solution.qs"
})
