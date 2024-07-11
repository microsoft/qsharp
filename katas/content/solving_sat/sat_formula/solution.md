This task consists of a conjunction (AND) of results of multiple clause evaluations. Each clause individually can be evaluated using the code you've written in the previous exercise. The computation results of these clauses must be stored temporarily in freshly allocated qubits. Then the conjunction of these results can be computed using `Oracle_And` from the first exercise.

Let's denote the number of clauses in the formula as $m$. The steps for implementing the SAT oracle will be:

1. Allocate an array of $m$ qubits `aux` in the state $\ket{0}$.
2. Evaluate each clause using `Oracle_SATClause` from the previous exercise, with the corresponding element of `aux` as the target qubit.
3. Evaluate the SAT formula using `Oracle_And` implemented in the first task with `aux` as the input register and `target` as the target qubit.
4. Undo step 2 to restore the auxiliary qubits back into the $\ket{0}$ state before releasing them.

You can again use the within-apply Q# language construct to perform steps 2, 3 and 4 with the last step generated automatically.

@[solution]({
    "id": "solving_sat__sat_formula_solution",
    "codePath": "./Solution.qs"
})
