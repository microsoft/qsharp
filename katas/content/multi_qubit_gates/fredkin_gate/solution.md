This is essentially bookwork, because there is only one gate that performs this state change (and the task title already gave it away!)
The Fredkin gate is also known as the controlled $SWAP$ gate:
$$
\begin{bmatrix}
1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
0 & 0 & 0 & 0 & 0 & 0 & 1 & 0 \\
0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
0 & 0 & 0 & 0 & 0 & 0 & 0 & 1
\end{bmatrix}
$$
and our initial state is:
$$
\begin{bmatrix}
\alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \zeta \\ \eta \\ \theta
\end{bmatrix}
$$
So we have:
$$
\begin{bmatrix}
1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
0 & 0 & 0 & 0 & 0 & 0 & 1 & 0 \\
0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
0 & 0 & 0 & 0 & 0 & 0 & 0 & 1
\end{bmatrix}
\begin{bmatrix}
\alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \color{blue}\zeta \\ \color{blue}\eta \\ \theta
\end{bmatrix} =
\begin{bmatrix}
\alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \color{red}\eta \\ \color{red}\zeta \\ \theta
\end{bmatrix} =
\alpha |000\rangle + \beta |001\rangle + \gamma |010\rangle + \delta |011\rangle + \epsilon |100\rangle + {\color{red}\eta}|101\rangle + {\color{red}\zeta}|110\rangle + \theta|111\rangle
$$

Notice carefully how the qubits are passed to the gate: `[qs[0]], (qs[1], [qs[2])`. The `Controlled` functor produces an operation that takes two parameters: the first one is an array of control qubits (in this case a single-element array consisting of the first qubit), and the second parameter is a tuple of all parameters you'd pass to the original gate (in this gate two individual qubit parameters that would be arguments to a $SWAP$ gate).

@[solution]({
    "id": "multi_qubit_gates__fredkin_gate_solution",
    "codePath": "./Solution.qs"
})
