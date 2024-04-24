This is essentially bookwork, because there is only one gate that performs this state change (and the task title already gave it away!) The Toffoli gate is:
$$
 \begin{bmatrix}1 & 0 & 0 & 0 & 0 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0\\\ 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0\\\ 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0\\\ 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1\\\ 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0\\\ \end{bmatrix}
$$
and our initial state is:
$$
\begin{bmatrix} \alpha\\\ \beta\\\ \gamma\\\ \delta\\\ \epsilon\\\ \zeta\\\ \eta\\\ \theta\\\ \end{bmatrix}
$$

So we have:

$$
\begin{bmatrix} 1 & 0 & 0 & 0 & 0 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0\\\ 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0\\\ 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0\\\ 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1\\\ 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0\\\ \end{bmatrix}
\begin{bmatrix} \alpha\\\ \beta\\\ \gamma\\\ \delta\\\ \epsilon\\\ \zeta\\\ \color{red}\theta\\\ \color{red}\eta\\\ \end{bmatrix}=

\begin{bmatrix} \alpha\\\ \beta\\\ \gamma\\\ \delta\\\ \epsilon\\\ \zeta\\\ \color{red}\theta\\\ \color{red}\eta\\\ \end{bmatrix}
\alpha |000\rangle + \beta |001\rangle + \gamma |010\rangle + \delta |011\rangle + \epsilon |100\rangle + \zeta|101\rangle + {\color{red}\theta}|110\rangle + {\color{red}\eta}|111\rangle
$$


@[solution]({
"id": "multi_qubit_gates__toffoli_gate_solution",
"codePath": "./Solution.qs"
})