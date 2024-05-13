#### Solution 1

We can recognize that the $S$ gate performs this particular relative phase addition to the $\ket{1}$ basis state. As a reminder,

$$
S =
\begin{bmatrix}1 & 0 \\ 0 & i\end{bmatrix}
$$

Let's see the effect of this gate on the general superposition $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$.

$$
 \begin{bmatrix}1 & 0 \\ 0 & i \end{bmatrix}
 \begin{bmatrix}\alpha \\ \beta \end{bmatrix}=
\begin{bmatrix}1\cdot\alpha + 0\cdot\beta \\ 0\cdot\alpha + i\cdot\beta \end{bmatrix}=
 \begin{bmatrix}\alpha \\ i\beta \end{bmatrix}
$$

It is therefore easy to see that when $\ket{\psi} = 0.6\ket{0} +  0.8\ket{1}, S\ket{\psi} =  0.6\ket{0} + 0.8i\ket{1}$.
@[solution]({
    "id": "single_qubit_gates__phase_i_solution_a",
    "codePath": "./SolutionA.qs"
})

#### Solution 2

Alternatively, see the Complex Relative Phase task later in the kata for an explanation of using $R1$ gate to implement the same transformation.

@[solution]({
    "id": "single_qubit_gates_phase_i_solution_b",
    "codePath": "./SolutionB.qs"
})
