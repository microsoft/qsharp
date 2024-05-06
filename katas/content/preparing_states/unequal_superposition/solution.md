We want to convert the $\ket{0}$ state to a parameterized superposition of $\ket{0}$ and $\ket{1}$, which suggests 
that we are looking for some kind of a rotation operation. There are three main gates that implement rotations around various axes of the Bloch Sphere: 

- $R_x(\theta) = \begin{bmatrix} \cos\frac{\theta}{2} & -i\sin\frac{\theta}{2} \\ -i\sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix}$
- $R_y(\theta) = \begin{bmatrix} \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\ \sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix}$
- $R_z(\theta) = \begin{bmatrix} e^{-i\theta/2} & 0 \\ 0 & e^{i\theta/2} \end{bmatrix}$

If we were to apply the $R_x$ gate to a qubit in the $|0\rangle$ state, we would introduce complex coefficients to the amplitudes, which is clearly not what we're looking for. Similarly, the $R_z$ gate introduces only a global phase when applied to $|0\rangle$ state, so we can rule it out as well. This leaves only the $R_y$ as a starting point for the solution.

Applying the $R_y$ gate to the $|0\rangle$ state, we get:
$$R_y(\theta) |0\rangle = 
\begin{bmatrix} \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\ \sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix} \begin{bmatrix} 1 \\ 0 \end{bmatrix} = 
\begin{bmatrix} \cos\frac{\theta}{2} \\ \sin\frac{\theta}{2} \end{bmatrix} = \cos\frac{\theta}{2}|0\rangle + \sin\frac{\theta}{2}|1\rangle$$

Therefore, applying the $R_y(2\alpha)$ gate to $|0\rangle$ is the solution to our problem. 

@[solution]({
    "id": "preparing_states__unequal_superposition_solution",
    "codePath": "./Solution.qs"
})
