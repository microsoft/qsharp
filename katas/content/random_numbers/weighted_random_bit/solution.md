An arbitrary single-qubit state can be written as:

$$
|\psi\rangle =
    \cos \frac{\theta}{2} |0 \rangle \, + \, e^{i\phi}  \sin \frac{\theta}{2} |1\rangle
$$

Here, $\theta$ is the angle between the state vector and the $Z$-axis, and $\phi$ is the longitude angle with respect to the $X$-axis on the Bloch sphere.

Our goal is to generate $0$ or $1$ with the probability of generating a $0$ equal to $x$ and the probability of generating a $1$ equal to $1 - x$. This means that the qubit state should look like

$$
|\psi\rangle =
    \sqrt x |0 \rangle + \sqrt{1 - x} |1\rangle
$$

Comparing the amplitudes of the state $|0 \rangle$ in the two equations, we get

$$
\sqrt x = \cos \frac{\theta}{2} \Rightarrow \theta = 2 \arccos\sqrt x
$$

Since $\theta$ is the angle between the state vector and the $Z$-axis, we need to apply the [`Ry`](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.ry) gate with the calculated $\theta$ to the starting state $|0 \rangle$ to get the desired qubit state.

The `Ry` operation applies a given rotation about the $Y$-axis (i.e., in the $ZX$-plane). Hence, $\phi$ (longitude angle with respect to $X$-axis) is always equal to $0^{\circ}$, which means that the relative phase $e^{i\phi}$ doesn't have any impact on the resulting qubit state.

> We can also calculate ${\theta}$ by comparing the amplitudes of the state $|1 \rangle$ in the two equations, which is $2 \arcsin\sqrt{1.0 - x}$

@[solution]({
    "id": "weighted_random_bit_solution",
    "codePath": "solution.qs"
})
