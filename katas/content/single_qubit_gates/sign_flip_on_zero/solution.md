The first thing to notice is that the gate $\begin{bmatrix} -1 & 0 \\ 0 & 1 \end{bmatrix}$ is quite similar to the Pauli $Z$ gate $\begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$.
The only difference being that the negative phase is applied on the $\ket{0}$ instead of $\ket{1}$. Hence you can simulate this gate by switching $\ket{0}$ and $\ket{1}$ states, applying the Pauli $Z$ gate and switching them back. The Pauli $X$ gate is the perfect gate to flip the state of the qubit and to undo the action afterwards.

Hence you can express the $Z_0 = \begin{bmatrix} -1 & 0 \\ 0 & 1 \end{bmatrix}$ matrix as

$$
Z_0 =
\begin{bmatrix} -1 & 0 \\ 0 & 1 \end{bmatrix} = 
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix} \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix} \begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix} = 
XZX
$$

@[solution]({
    "id": "single_qubit_gates__sign_flip_on_zero_solution",
    "codePath": "./Solution.qs"
})
