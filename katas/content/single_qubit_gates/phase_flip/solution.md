#### Solution 1

We can recognize that the S gate performs this particular relative phase addition to the $|1\rangle$ basis state. As a reminder,

$$
S =
\begin{bmatrix}1 & 0\\\ 0 & i\end{bmatrix}
$$

Let's see the effect of this gate on the general superposition $|\psi\rangle = \alpha |0\rangle + \beta |1\rangle$.

$$
 \begin{bmatrix}1 & 0 \\\ 0 & i \end{bmatrix}
 \begin{bmatrix}\alpha\\\ \beta\\\ \end{bmatrix}=
\begin{bmatrix}1\cdot\alpha + 0\cdot\beta\\\ 0\cdot\alpha + i\cdot\beta \end{bmatrix}=
 \begin{bmatrix}\alpha\\\ i\beta\\\ \end{bmatrix}
$$

It is therefore easy to see that when $|\psi\rangle = 0.6|0\rangle +  0.8|1\rangle, S|\psi\rangle =  0.6|0\rangle + 0.8i|1\rangle$.
@[solution]({
"id": "single_qubit_gates__phase_flip_solution_a",
"codePath": "./SolutionA.qs"
})

#### Solution 2

Alternatively, see the Phase Change task later in the kata for an explanation of using R1 gate to implement the same transformation.

@[solution]({
"id": "single_qubit_gates_phase_flip_solution_b",
"codePath": "./SolutionB.qs"
})
