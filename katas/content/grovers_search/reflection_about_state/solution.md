You know that $P\ket{0...0} = \ket{\psi}$.
You can use this and the fact that $I = PP^\dagger$ to write the following representation of the operation you need to implement:

$$2\ket{\psi}\bra{\psi} - I = 2P\ket{0...0}\bra{0...}P^\dagger - I =$$
$$= 2P\ket{0...0}\bra{0...0}P^\dagger - PP^\dagger = P(2\ket{0...0}\bra{0...0} - I)P^\dagger$$

The middle operation is exactly the conditional phase flip you implemented in the previous exercise.
The overall solution is:

1. Apply adjoint of the given operation $P$.
2. Apply the conditional phase flip from the previous exercise.
3. Apply the given operation $P$.

@[solution]({
    "id": "grovers_search__reflection_about_state_solution",
    "codePath": "Solution.qs"
})
