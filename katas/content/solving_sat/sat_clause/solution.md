This task involves evaluating a clause which is a disjunction (OR) of negated and non-negated variables encoded in the input $x$. 
This can be done in two steps:

1. First, flip the qubits which are negated in `clause` (and later undo this operation).  
   To do this, apply an $X$ gate to qubit $j$ if and only if the clause has a term of the form `(j, false)`.
2. Use the $U_{or}$ unitary (implemented by the operation `Oracle_Or`) to calculate the clause.  
   To do this, you need to first construct an array `clauseQubits` - all qubits which are included as a negated or non-negated variable in the clause. Then, you can apply the `Oracle_Or` operation to qubits `clauseQubits` and `y`.

Note that the implementation of `Oracle_SATClause` should be adjointable, and it's nice to be able to rely on Q# compiler to generate the adjoint variant of this operation.
That's why the solution uses the library function `Mapped` instead of classical computations that involve manipulating mutable variables - those would have to be moved to a separate function, making the code bulkier.

@[solution]({
    "id": "solving_sat__sat_clause_solution",
    "codePath": "./Solution.qs"
})
