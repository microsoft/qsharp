The action of the Pauli Z gate is exactly what is required by this question.
This gate leaves the sign of the $|0\rangle$ component of the superposition unchanged but flips the sign of the $|1\rangle$ component of the superposition.

As a reminder, the Pauli Z gate is defined by the following matrix:

$$
Z =
 \begin{bmatrix}1 & 0\\\0 & -1 \end{bmatrix}
$$

Let's see its effect on the only computational basis state that it changes, $|1\rangle$:

$$
Z|1\rangle =
 \begin{bmatrix} 1 & 0\\\0 & -1 \end{bmatrix}
 \begin{bmatrix}0\\\ 1\\\ \end{bmatrix}=
\begin{bmatrix}1 \cdot 0 + 0 \cdot1\\\0 \cdot 1 +  -1 \cdot 1\\\ \end{bmatrix}=
\begin{bmatrix}0\\\ -1\\\ \end{bmatrix}=
 -\begin{bmatrix}0\\\ 1\\\ \end{bmatrix}=
-|1\rangle
$$

In general applying the Z gate to a single qubit superposition state $|\psi\rangle = \alpha |0\rangle + \beta |1\rangle$ gives

$$
Z|\psi\rangle =
 \begin{bmatrix}1 & 0 \\\0 & -1\end{bmatrix}
 \begin{bmatrix}\alpha\\\\beta\\\ \end{bmatrix}=
\begin{bmatrix}1\cdot\alpha + 0\cdot\beta\\\0\cdot\alpha + -1\cdot\beta\\\ \end{bmatrix}=
 \begin{bmatrix}\alpha\\\ -\beta\\\ \end{bmatrix}=
 \alpha |0\rangle -\beta |1\rangle
$$

@[solution]({
"id": "single_qubit_gates__sign_flip_solution",
"codePath": "./Solution.qs"
})
