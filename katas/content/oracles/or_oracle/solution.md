What if you needed to flip the state of the target qubit only if the input register was in the $|0...0\rangle$ state?
In that case, you could use `ApplyControlledOnInt` with the control pattern $0$ that corresponds to all control qubits in $|0\rangle$ state.

In this problem, you need to separate the $|0...0\rangle$ basis state from all others, but with the opposite effect on the target qubit: instead of flipping it for this input state, you need to flip it for all other input states.
Or, you can think of it as first flipping the state of the target qubit for all states, and then un-flipping it (or flipping it again) for just this basis state. You can do this by applying the $X$ gate before or after `ApplyControlledOnInt`.

Notice that you can modify the state of the input register during your computations (this is what `ApplyControlledOnInt` function does under the hood). However, it is essential to undo those modifications ("uncompute" the changes), except the final one, so that the oracle will preserve the input if it is a basis state.

@[solution]({
    "id": "oracles__or_oracle_solution",
    "codePath": "Solution.qs"
})
