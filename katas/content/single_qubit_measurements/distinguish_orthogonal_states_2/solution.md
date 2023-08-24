We can distinguish between the states $\ket{A}$ and $\ket B$ if we implement a measurement in the $\{ \ket{A}, \ket{B}\}$ basis.

We can notice that the $R_x$ rotation gate with $\theta = 2 \alpha$ is an appropriate transformation which maps the $\ket 0 $ state to the $\ket A$ state, and the $\ket 1$ state to the $\ket B$ state:

$$R_x(\theta) \ket 0 = \cos \alpha \ket 0 -i \sin \alpha \ket 1 = \ket {A},$$
$$R_x(\theta) \ket 1 = -i \sin \alpha \ket 0 + \cos \alpha \ket 1 = \ket{B}.$$

Thus, the inverse transformation $R_x(-\theta)$ maps the $A/B$ basis to the $0/1$ basis.

Therefore, if we apply $R_x(-\theta)$ to the qubit and measure it using `M`, a measurement result of `Zero` will correspond to the qubit's original state being $\ket{A}$, while a result of `One` will correspond to the qubit's original state being $\ket B$.

@[solution]({
    "id": "distinguish_orthogonal_states_2_solution",
    "exerciseId": "distinguish_orthogonal_states_2",
    "codePath": "solution.qs"
})
