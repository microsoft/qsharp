The first thing to notice is that the gate $\begin{bmatrix} -1 & 0 \\\ 0 & 1 \end{bmatrix}$ is quite similar to the Pauli $Z$ gate $\begin{bmatrix} 1 & 0 \\\ 0 & -1 \end{bmatrix}$.
The only difference being that the negative phase is applied on the $|0\rangle$ instead of $|1\rangle$. Hence we can simulate this gate by switching $|0\rangle$ and $|1\rangle$ states, applying the Pauli $Z$ gate and switching them back. The Pauli $X$ gate (also called the $NOT$ gate or the bit flip gate) is the perfect gate to flip the state of the qubit and to undo the action afterwards.

Hence we can express the $Z_0 = \begin{bmatrix} -1 & 0 \\\ 0 & 1 \end{bmatrix}$ matrix as

$$
Z_0 =
\begin{bmatrix} -1 & 0 \\\ 0 & 1 \end{bmatrix} = 
\begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix} \begin{bmatrix} 1 & 0 \\\ 0 & -1 \end{bmatrix} \begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix} = 
XZX
$$

@[solution]({
    "id": "sign_flip_on_zero_solution",
    "codePath": "./Solution.qs"
})
