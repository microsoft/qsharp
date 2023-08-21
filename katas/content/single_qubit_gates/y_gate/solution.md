We have to do exactly what the task asks us to do: apply the Pauli gate $Y=\\begin{bmatrix} 0 & -i \\\\ i & 0 \\end{bmatrix}$.

This has the effect of turning $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$ into $Y|\psi\rangle = i\alpha|1\rangle - i\beta|0\rangle$, which in matrix form looks as follows:
$$
\begin{bmatrix} 0 & -i \\\ i & 0 \end{bmatrix} \begin{bmatrix} \alpha \\\ \beta \end{bmatrix} = 
\begin{bmatrix} -i\beta \\\ i\alpha \end{bmatrix}
$$

@[solution]({
    "id": "y_gate_solution",
    "codePath": "./Solution.qs"
})
