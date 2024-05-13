The easiest solution is to transform the given state so that the basis state that matches the given pattern becomes $\ket{1...1}$,
apply the Controlled $Z$ gate to flip the phase of just that basis state, and then uncompute to make sure that only the relative phases of the basis states change, not the basis states themselves. 

You can use the `within ... apply` construct to automate uncomputation.

@[solution]({
    "id": "oracles__bit_pattern_challenge_solution",
    "codePath": "Solution.qs"
})
