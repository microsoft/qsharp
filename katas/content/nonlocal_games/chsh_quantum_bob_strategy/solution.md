Measuring a qubit in the $\theta$ basis is the same as rotating the qubit by $\theta$, clockwise, and then making a standard measurement in the Z basis.

To implement the described transformation in Q#, we need to rotate the qubit by $\frac{\pi}{8}$ clockwise if `bit = false` or counterclockwise if `bit = true` and then perform a measurement.
We can do the rotation using the previous task (note the negation of the boolean parameter we need to do).

(See the lesson below for details on why Bob should follow this strategy.)

@[solution]({
    "id": "nonlocal_games__chsh_quantum_bob_strategy_solution",
    "codePath": "Solution.qs"
})
