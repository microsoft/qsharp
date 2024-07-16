As we saw in the Oracles kata, we can allocate an additional qubit in the $\ket{-}$ state and use it as the target for our marking oracle.
This will kick back the $-1$ relative phase for the basis states $\ket{x}$ of the input register for which $f(x) = 1$.

@[solution]({
    "id": "grovers_search__phase_oracle_solution",
    "codePath": "Solution.qs"
})
