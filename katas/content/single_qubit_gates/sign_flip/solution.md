The action of the Pauli Z gate is exactly what is required by this question.
This gate leaves the sign of the $\ket{0}$ component of the superposition unchanged but flips the sign of the $\ket{1}$ component of the superposition.

As a reminder, the Pauli Z gate is defined by the following matrix:

$$
Z =
 \begin{bmatrix}1 & 0 \\ 0 & -1 \end{bmatrix}
$$

Let's see its effect on the only computational basis state that it changes, $\ket{1}$:

$$
Z\ket{1} =
 \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}
 \begin{bmatrix}0 \\ 1 \end{bmatrix}=
\begin{bmatrix}1 \cdot 0 + 0 \cdot1 \\ 0 \cdot 1 +  (-1) \cdot 1 \end{bmatrix}=
\begin{bmatrix}0 \\ -1 \end{bmatrix}=
 -\begin{bmatrix}0 \\ 1 \end{bmatrix}=
-\ket{1}
$$

In general applying the Z gate to a single qubit superposition state $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$ gives

$$
Z\ket{\psi} =
 \begin{bmatrix}1 & 0 \\ 0 & -1 \end{bmatrix}
 \begin{bmatrix}\alpha \\ \beta \end{bmatrix}=
\begin{bmatrix}1\cdot\alpha + 0\cdot\beta \\ 0\cdot\alpha + (-1)\cdot\beta \end{bmatrix}=
 \begin{bmatrix}\alpha \\ -\beta \end{bmatrix}=
 \alpha \ket{0} -\beta \ket{1}
$$

@[solution]({
"id": "single_qubit_gates__sign_flip_solution",
"codePath": "./Solution.qs"
})
