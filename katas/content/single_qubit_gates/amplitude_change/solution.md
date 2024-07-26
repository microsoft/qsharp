You can recognize that you need to use one of the rotation gates Rx, Ry, and Rz (named because they "rotate" the qubit state in the three dimensional space visualized as the Bloch sphere about the x, y, and z axes, respectively), since they involve angle parameters. Of these three gates, only Ry rotates the basis states $\ket{0}$ and $\ket{1}$ to have real amplitudes (the other two gates introduce complex coefficients).

As a reminder,

$$
R_{y}(\theta) =
\begin{bmatrix}\cos \frac{\theta}{2} & -\sin \frac{\theta}{2}\\ \sin \frac{\theta}{2} & \cos \frac{\theta}{2}\end{bmatrix}
$$

Let's see its effect on the $\ket{0}$ state:

$$
R_y(\theta)\ket{0} =
\begin{bmatrix}\cos \frac{\theta}{2} & -\sin \frac{\theta}{2}\\ \sin \frac{\theta}{2} & \cos \frac{\theta}{2} \end{bmatrix}
\begin{bmatrix}1\\ 0\\ \end{bmatrix}=
\begin{bmatrix}\cos \frac{\theta}{2}\cdot1 - \sin \frac{\theta}{2}\cdot0\\ \sin \frac{\theta}{2}\cdot1 + \cos \frac{\theta}{2}\cdot0
\end{bmatrix}=
\begin{bmatrix}\cos \frac{\theta}{2}\\ \sin \frac{\theta}{2}\end{bmatrix}=
\cos\frac{\theta}{2} \ket{0} + \sin\frac{\theta}{2} \ket{1}
$$

Recall that when applying a gate, you can tell what its matrix does to the basis states by looking at its columns: the first column of the matrix is the state into which it will transform the $\ket{0}$ state, and the second column is the state into which it will transform the $\ket{1}$ state.
One of the examples used by the testing harness has $\beta = 0.6, \gamma = 0.8$ and $\alpha = \frac{\pi}{3} ≈ 1.0472$.
Since $\cos \frac{\pi}{3} = 0.5$ and $\sin \frac{\pi}{3} = 0.8660$, working to 4 decimal places, you can use $\frac{\theta}{2} = \alpha$ to compute:

$$
R_{y}(\theta) \ket{\psi}=
 \begin{bmatrix}\cos \frac{\theta}{2} & -\sin \frac{\theta}{2}\\ \sin \frac{\theta}{2} & \cos \frac{\theta}{2} \end{bmatrix}
  \begin{bmatrix}\beta\\ \gamma\\ \end{bmatrix}=
  \begin{bmatrix}\cos \frac{\theta}{2}\cdot\beta - \sin \frac{\theta}{2}\cdot\gamma\\ \sin \frac{\theta}{2}\cdot\beta +\cos \frac{\theta}{2}\cdot\gamma \end{bmatrix}=
 \begin{bmatrix} 0.6\cdot\cos \frac{\pi}{3} -0.8\cdot\sin \frac{\pi}{3}\\0.6\cdot\sin \frac{\pi}{3} +0.8\cdot\cos \frac{\pi}{3}\end{bmatrix}=
 \begin{bmatrix}0.3 - 0.6928\\ 0.5196 + 0.4\end{bmatrix}=
\begin{bmatrix}-0.3928\\ 0.9196\\ \end{bmatrix}
$$

Notice that $\frac{\theta}{2} = \alpha$; this means that in the Q# code you need to pass the angle $\theta = 2\alpha$.

@[solution]({
"id": "single_qubit_gates__amplitude_change_solution",
"codePath": "./Solution.qs"
})
