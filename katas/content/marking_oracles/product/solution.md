This problem is similar to the previous one, but this time the input qubit `x[j]` affects the state of the target qubit only if the classical input `r[i]` is set to `true`. 
We can use a similar approach to the solution as well: iterate through all input qubits, check whether the corresponding input bit `r[i]` is `true`, and if it is, apply a $CNOT$ gate with qubit `x[j]` as the control and the qubit `y` as the target.

@[solution]({
    "id": "marking_oracles__product_solution",
    "codePath": "./Solution.qs"
})
