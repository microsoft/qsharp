You want to convert the $\ket{0}$ state to a parameterized superposition of $\ket{0}$ and $\ket{1}$, which suggests 
that you're looking for some kind of a rotation operation. There are three main gates that implement rotations around various axes of the Bloch Sphere: 

- $R_x(\theta) = \begin{bmatrix} \cos\frac{\theta}{2} & -i\sin\frac{\theta}{2} \\ -i\sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix}$
- $R_y(\theta) = \begin{bmatrix} \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\ \sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix}$
- $R_z(\theta) = \begin{bmatrix} e^{-i\theta/2} & 0 \\ 0 & e^{i\theta/2} \end{bmatrix}$

If you were to apply the $R_x$ gate to a qubit in the $\ket{0}$ state, you'd introduce complex coefficients to the amplitudes, which is clearly not what you're looking for. Similarly, the $R_z$ gate introduces only a global phase when applied to $\ket{0}$ state, so you can rule it out as well. This leaves only the $R_y$ as a starting point for the solution.

Applying the $R_y$ gate to the $\ket{0}$ state, you get:
$$R_y(\theta) \ket{0} = 
\begin{bmatrix} \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\ \sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix} \begin{bmatrix} 1 \\ 0 \end{bmatrix} = 
\begin{bmatrix} \cos\frac{\theta}{2} \\ \sin\frac{\theta}{2} \end{bmatrix} = \cos\frac{\theta}{2}\ket{0} + \sin\frac{\theta}{2}\ket{1}$$

Therefore, applying the $R_y(2\alpha)$ gate to $\ket{0}$ is the solution to this problem. 

@[solution]({
    "id": "preparing_states__unequal_superposition_solution",
    "codePath": "./Solution.qs"
})
