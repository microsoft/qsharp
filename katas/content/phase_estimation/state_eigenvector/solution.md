A quantum state is an eigenstate of a quantum gate if applying that gate to that state doesn't change it, other than multiply it by a global phase. This means that your solution should probably start by preparing the state $\ket{\psi}$ and applying the unitary $U$ to it. How can you check that the state after that is still $\ket{\psi}$ (up to a global phase)?

Let's consider what happens if you apply the adjoint of $P$ to the state $\ket{\psi}$:

$$P^\dagger \ket{\psi} = P^\dagger P\ket{0} = I\ket{0} = \ket{0}$$

You can use this to finish the solution: apply `Adjoint P` to the state you obtained after applying $U$ and check whether the result is $\ket{0}$ using the library operation `CheckZero`.

@[solution]({
    "id": "phase_estimation__state_eigenvector_solution", 
    "codePath": "Solution.qs"
})
