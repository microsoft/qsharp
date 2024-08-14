This is essentially bookwork, because there's only one gate that performs this state change (and the task title already gave it away!) The Toffoli gate is:
$$
 \begin{bmatrix}
 1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\
 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
 \end{bmatrix}
$$
and your initial state is:
$$
\begin{bmatrix} \alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \zeta \\ \eta \\ \theta
\end{bmatrix}
$$

So you have:

$$
 \begin{bmatrix}
 1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\
 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
 \end{bmatrix}
 \begin{bmatrix}
 \alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \zeta \\ \color{blue}\eta \\ \color{blue}\theta
 \end{bmatrix}=
 \begin{bmatrix}
 \alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \zeta \\ \color{red}\theta \\ \color{red}\eta
 \end{bmatrix}=
\alpha \ket{000} + \beta \ket{001} + \gamma \ket{010} + \delta \ket{011} + \epsilon \ket{100} + \zeta\ket{101} + {\color{red}\theta}\ket{110} + {\color{red}\eta}\ket{111}
$$

@[solution]({
    "id": "multi_qubit_gates__toffoli_gate_solution",
    "codePath": "./Solution.qs"
})
