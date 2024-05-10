We can recognize that the Hadamard gate changes states $\ket{0}$ and $\ket{1}$ to $\ket{+}$ and $\ket{-}$, respectively, and vice versa.

As a reminder, the Hadamard gate is defined by the following matrix:

$$
\frac{1}{\sqrt{2}}\begin{bmatrix}1 & 1 \\1 & -1\end{bmatrix}
$$

For example, we can work out $H\ket{1}$ as follows:

$$
H\ket{1}=
\frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\1 & -1\end{bmatrix}
\begin{bmatrix} 0\\ 1\end{bmatrix}=
\frac{1}{\sqrt{2}}\begin{bmatrix}1 \cdot 0 + 1 \cdot 1 \\1 \cdot 0 + (-1) \cdot 1\end{bmatrix}=
\frac{1}{\sqrt{2}}\begin{bmatrix}1\\ -1\\ \end{bmatrix}=
\frac{1}{\sqrt{2}} \big(\ket{0} - \ket{1}\big) = \ket{-}
$$

Similarly, we can consider the effect of the Hadamard gate on the superposition state $\ket{\psi} = 0.6\ket{0} + 0.8\ket{1}$ (rounding the numbers to 4 decimal places):

$$
H|\psi⟩ =
\frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix}
 \begin{bmatrix} \alpha\\ \beta\\ \end{bmatrix} =
\frac{1}{\sqrt{2}}\begin{bmatrix} \alpha + \beta\\ \alpha - \beta\\ \end{bmatrix}=
0.7071\begin{bmatrix} 1.4\\ -0.2\\ \end{bmatrix} =
\begin{bmatrix}
   0.98994\\ -0.14142\\ \end{bmatrix} =
   0.9899\ket{0} - 0.1414\ket{1}
$$

@[solution]({
"id": "single_qubit_gates__basis_change_solution",
"codePath": "./Solution.qs"
})
