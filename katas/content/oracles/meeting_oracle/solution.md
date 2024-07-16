In this problem we'll need to allocate extra qubits, one per each day of the week, to serve as temporary variables.
Each of these variables will track whether both people are free on that day, that is, whether both input qubit arrays 
have bit $0$ in the corresponding position. We can compute each of these variables using a $CCNOT$ gate after flipping 
the states of both qubits using $X$ gates. 

After this, we can reuse the marking OR oracle to check whether any of the days are in $\ket{1}$ state.

When you allocate qubits for temporary use during a larger computation, you need to make sure they are returned to the $\ket{0}$ 
state before being released. Unlike your previous experience allocating qubits in demos, though, this time you cannot 
measure these qubits for this - they are likely entangled with the qubits you're still using, so a measurement would 
collapse the state of the whole system, not just the auxiliary qubits. Instead, you have to uncompute the state of these qubits, 
similar to how you uncompute any changes you do to the input qubits to be able to apply controlled gates for the right pattern.
`within ... apply` construct is helpful here as well.

@[solution]({
    "id": "oracles__meeting_oracle_solution",
    "codePath": "Solution.qs"
})
