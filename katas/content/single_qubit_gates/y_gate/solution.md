You have to do exactly what the task asks us to do: apply the Pauli gate $Y=\begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix}$.

This has the effect of turning $\ket{\psi} = \alpha\ket{0} + \beta\ket{1}$ into $Y\ket{\psi} = i\alpha\ket{1} - i\beta\ket{0}$, which in matrix form looks as follows:
$$
\begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix} \begin{bmatrix} \alpha \\ \beta \end{bmatrix} = 
\begin{bmatrix} -i\beta \\ i\alpha \end{bmatrix}
$$

@[solution]({
    "id": "single_qubit_gates__y_gate_solution",
    "codePath": "./Solution.qs"
})
