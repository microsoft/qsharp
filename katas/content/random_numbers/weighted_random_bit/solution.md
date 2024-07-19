An arbitrary single-qubit state can be written as:

$$
\ket{\psi} =
    \cos \frac{\theta}{2} \ket{0 } + e^{i\phi}  \sin \frac{\theta}{2} \ket{1}
$$

Here, $\theta$ is the angle between the state vector and the $Z$-axis, and $\phi$ is the longitude angle with respect to the $X$-axis on the Bloch sphere.

Your goal is to generate $0$ or $1$ with the probability of generating a $0$ equal to $x$ and the probability of generating a $1$ equal to $1 - x$. This means that the qubit state should look like

$$
\ket{\psi} =
    \sqrt x \ket{0 } + \sqrt{1 - x} \ket{1}
$$

Comparing the amplitudes of the state $\ket{0 }$ in the two equations, you get

$$
\sqrt x = \cos \frac{\theta}{2} \Rightarrow \theta = 2 \arccos\sqrt x
$$

Since $\theta$ is the angle between the state vector and the $Z$-axis, you need to apply the $Ry$ gate with the calculated $\theta$ to the starting state $\ket{0 }$ to get the desired qubit state.

The $Ry$ gate applies a given rotation about the $Y$-axis, that is, in the $ZX$-plane. Hence, $\phi$ (longitude angle with respect to $X$-axis) is always equal to $0^{\circ}$, which means that the relative phase $e^{i\phi}$ doesn't have any impact on the resulting qubit state.

> You can also calculate ${\theta}$ by comparing the amplitudes of the state $\ket{1 }$ in the two equations, which is $2 \arcsin\sqrt{1.0 - x}$. The results will be equivalent.

@[solution]({
    "id": "random_numbers__weighted_random_bit_solution",
    "codePath": "Solution.qs"
})
