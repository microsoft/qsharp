The key observation here is that $R_z$ is a diagonal matrix and $R_y$ is not, so when applied to the $\ket{0}$ state, the former will leave it unchanged (with an extra phase which is not observable), and the latter will convert it to a superposition $\cos\frac{\theta}{2} \ket{0} + \sin\frac{\theta}{2} \ket{1}$. The question is, how to distinguish those two states if they are not orthogonal (and for most values of $\theta$ they will not be)?

The task description gives you a big hint: it allows you to use the given unitary unlimited number of times, which points to a probabilistic solution (as opposed to deterministic solutions in all previous problems in this kata). Apply the unitary to the $\ket{0}$ state and measure the result; if it is $\ket{1}$, the unitary must be $R_y$, otherwise you can repeat the experiment again. After several iterations of measuring $\ket{0}$ you can conclude that with high probability the unitary is $R_z$.

To reduce the number of iterations after which you make the decision, you could apply the unitary several times to bring the overall rotation angle closer to $\pi$: in case of $R_y$ gate this would allow you to rotate the state closer to the $\ket{1}$ state, so that you'd detect it with higher probability.

@[solution]({
    "id": "distinguishing_unitaries__rz_ry_solution",
    "codePath": "Solution.qs"
})
