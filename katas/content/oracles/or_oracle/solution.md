@[solution]({
    "id": "or_oracle_solution",
    "codePath": "solution.qs"
})

Notice that you can modify the state of the input register during your computations (this is what `ControlledOnInt` function does under the hood). However, it is essential to undo those modifications ("uncompute" the changes), except the final one, so that the oracle will preserve the input if it is a basis state.
