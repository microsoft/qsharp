You can recognize that the Pauli $X$ gate will change the state $\ket{0}$ to $\ket{1}$ and vice versa, and $\alpha\ket{0} + \beta\ket{1}$ to $\alpha\ket{1} + \beta\ket{0}$.

As a reminder, the Pauli $X$ gate is defined by the following matrix:

$$
X =
\begin{bmatrix} 0 &  1 \\ 1 &  0 \end{bmatrix}
$$

You can see how it affects, for example, the basis state $\ket{0}$:

$$
X\ket{0} =
\begin{bmatrix} 0 &  1 \\ 1 &  0 \end{bmatrix}
\begin{bmatrix} 1 \\ 0 \end{bmatrix} =
\begin{bmatrix} 0 \cdot 1 + 1 \cdot 0 \\ 1 \cdot 1 + 0 \cdot 0 \end{bmatrix}=
\begin{bmatrix} 0 \\1 \end{bmatrix}=
\ket{1}
$$

Similarly, you can consider the effect of the $X$ gate on the superposition state $\ket{\psi} = 0.6\ket{0} + 0.8\ket{1}$:

$$
X\ket{\psi} =
\begin{bmatrix} 0 &  1 \\1 & 0\end{bmatrix}
\begin{bmatrix}0.6 \\ 0.8 \end{bmatrix}=
\begin{bmatrix} 0 \cdot 0.6 + 1 \cdot 0.8 \\ 1 \cdot 0.6 + 0 \cdot 0.8 \end{bmatrix}=
\begin{bmatrix}0.8 \\ 0.6 \end{bmatrix}=
0.8\ket{0} + 0.6\ket{1}
$$

@[solution]({
"id": "single_qubit_gates__state_flip_solution",
"codePath": "./Solution.qs"
})
