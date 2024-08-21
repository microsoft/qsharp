This is essentially bookwork, because there's only one gate that performs this state change (and the task title already gave it away!)
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
and the initial state is:
$$
\begin{bmatrix}
\alpha \\ \beta \\ \gamma \\ \delta \\ \epsilon \\ \zeta \\ \eta \\ \theta
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
\alpha \ket{000} + \beta \ket{001} + \gamma \ket{010} + \delta \ket{011} + \epsilon \ket{100} + {\color{red}\eta}\ket{101} + {\color{red}\zeta}\ket{110} + \theta\ket{111}
$$

Notice carefully how the qubits are passed to the gate: `[qs[0]], (qs[1], [qs[2])`. The `Controlled` functor produces an operation that takes two parameters: the first one is an array of control qubits (in this case a single-element array consisting of the first qubit), and the second parameter is a tuple of all parameters you'd pass to the original gate (in this gate two individual qubit parameters that would be arguments to a $SWAP$ gate).

@[solution]({
    "id": "multi_qubit_gates__fredkin_gate_solution",
    "codePath": "./Solution.qs"
})
