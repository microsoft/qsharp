We can distinguish between the states $\ket{\psi_\pm}$ if we implement a measurement in the $\{ \ket{\psi_+}, \ket{\psi_-}\}$ basis. This can be done if we construct a unitary transformation which maps the $\ket{\psi_+}$ state to the $\ket{0}$ state, and the $\ket{\psi_{-}}$ state to the $\ket{1}$ state.

We can notice that the $R_y$ rotation gate with $\theta = 2 \arctan \frac{0.8}{0.6}$ is an appropriate transformation:

$$R_y(\theta) \ket 0 = 0.6 \ket 0 + 0.8 \ket 1 = \ket {\psi_+},$$
$$R_y(\theta) \ket 1 = -0.8 \ket 0 + 0.6 \ket 1 = \ket{\psi_-}.$$

Thus, the inverse (adjoint) transformation $R_y(-\theta)$ maps the $\ket{\psi_\pm}$ basis to the computational basis, i.e.,
$$R_y(-\theta) \ket {\psi_+} = \ket 0,$$
$$R_y(-\theta) \ket {\psi_-} = \ket 1.$$

Hence, if we apply $R_y(-\theta)$ to the qubit, its state will be transformed to one of the computational basis states, at which point we can measure it using `M`. If `M` returns `Zero`, the rotated state is $\ket{0}$, which means that the original state of the qubit was $\ket{\psi_+}$. Similarly, an output of `One` indicates that the qubit was originally in the state $\ket{\psi_-}$.

@[solution]({
    "id": "distinguish_orthogonal_states_1_solution",
    "exerciseId": "distinguish_orthogonal_states_1",
    "codePath": "solution.qs"
})
