We can recognize that the Hadamard gate changes states $|0\rangle$ and $|1\rangle$ to $|+\rangle$ and $|-\rangle$, respectively, and vice versa.

As a reminder, the Hadamard gate is defined by the following matrix:

$$
\frac{1}{\sqrt{2}}\begin{bmatrix}1 & 1 \\\1 & -1\end{bmatrix}
$$

For example, we can work out $H|1\rangle$ as follows:

$$
H|1\rangle=
\frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\\1 & -1\end{bmatrix}
\begin{bmatrix} 0\\\ 1\end{bmatrix}=
\frac{1}{\sqrt{2}}\begin{bmatrix}1 \cdot 0 + 1 \cdot 1 \\\1 \cdot 0 + (-1) \cdot 1\end{bmatrix}=
\frac{1}{\sqrt{2}}\begin{bmatrix}1\\\ -1\\\ \end{bmatrix}=
\frac{1}{\sqrt{2}} \big(|0\rangle - |1\rangle\big) = |-\rangle
$$

Similarly, we can consider the effect of the Hadamard gate on the superposition state $|\psi\rangle = 0.6|0\rangle + 0.8|1\rangle$ (rounding the numbers to 4 decimal places):

$$
H|\psi⟩ =
\frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix}
 \begin{bmatrix} \alpha\\\ \beta\\\ \end{bmatrix} =
\frac{1}{\sqrt{2}}\begin{bmatrix} \alpha + \beta\\\ \alpha - \beta\\\ \end{bmatrix}=
0.7071\begin{bmatrix} 1.4\\\ -0.2\\\ \end{bmatrix} =
\begin{bmatrix}
   0.98994\\\ -0.14142\\\ \end{bmatrix} =
   0.9899|0\rangle - 0.1414|1\rangle
$$

@[solution]({
"id": "single_qubit_gates__basis_change_solution",
"codePath": "./Solution.qs"
})
