Both input states are superposition states, with equal absolute values of amplitudes of both basis states. This means if the sate is measured in the Pauli $Z$ basis, there is a 50-50 chance of measuring `One` or `Zero`, which won't give us the necessary information.

To determine in which state the input qubit is with certainty, we want to transform the qubit into a state where there is no superposition with respect to the basis in which we perform the measurement.

Consider how we can prepare the input states, starting with basis states: $H\ket{0} = \ket{+}$ and $H\ket{1} = \ket{-}$. This transformation can also be undone by applying the $H$ gate again (remember that the $H$ gate is self-adjoint, i.e., it equals its own inverse): $H\ket{+} = \ket{0}$ and $H\ket{-} = \ket{1}$.

Once we have the $\ket{0}$ or $\ket{1}$ state, we can use the same principle as in previous task $\ket{0}$ or $\ket{1}$ to measure the state and report the outcome. Note that in this task return value `true` corresponds to input state $\ket{+}$, so we compare the measurement result with `Zero`.

@[solution]({
    "id": "distinguishing_states__plus_minus_solution_a",
    "codePath": "SolutionA.qs"
})

#### Alternative solution

Another possible solution could be to measure in the Pauli $X$ basis ($\ket{+}, \ket{-}$ basis), this means a transformation with the $H$ gate before measurement is not needed. Again, measurement result `Zero` would correspond to state $\ket{+}$.

In Q#, measuring in another Pauli basis can be done with the `Measure()` operation.

@[solution]({
    "id": "distinguishing_states__plus_minus_solution_b",
    "codePath": "SolutionB.qs"
})
