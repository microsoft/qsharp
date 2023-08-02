We already learnt how to generate random bit with equal probability in exercise 1, in this exercise we need to generate random bit with weighted probability.

An arbitrary single-qubit state can be written as:

$$
|\psi\rangle =
    \cos \frac{\theta}{2} |0 \rangle \, + \, e^{i\phi}  \sin \frac{\theta}{2} |1\rangle
$$

Here $\theta$ is angle between state vector and $Z$-axis, and $\phi$ is longitude angle with respect to $X$-axis on the Bloch sphere.

Our goal is to generate 0 or 1 with probability of 0 equal to $x$ and probability of 1 equal to $1 - x$, which means the qubit state should look like

$$
|\psi\rangle =
    \sqrt x |0 \rangle + \sqrt{1 - x} |1\rangle
$$

By comparing the amplitudes of the state $|0 \rangle$ on both equations we get

$$
\sqrt x = \cos \frac{\theta}{2} \Rightarrow \theta = 2 \arccos\sqrt x
$$

Since $\theta$ is angle between state vector and the $Z$-axis, we need to apply the [Ry](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.ry) gate with caculated $\theta$ to the starting state $|0 \rangle$ to get the desired qubit state.

Ry operation applies a given rotation about $Y$-axis (i.e., in the $ZX$-plane), hence $\phi$ (longitude angle with respect to $X$-axis) is always equal to $0^{\circ}$, which means that the relative phase $e^{i\phi}$ doesn't have any impact on resulting qubit state.

> We can also calculate ${\theta}$ by comparing the amplitudes of the state $|1 \rangle$ on both equations, which is $2 \arcsin\sqrt{1.0 - x}$

@[solution]({
    "id": "weighted_random_bit_solution",
    "codePath": "solution.qs"
})
