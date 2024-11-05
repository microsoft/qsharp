Firstly, notice that you're dealing with an unentangled pair of qubits.
In vector form the transformation you need is
$$
\frac{1}{2}\begin{bmatrix}1 \\ 1 \\ 1 \\ 1 \end{bmatrix}
\rightarrow
\frac{1}{2}\begin{bmatrix}1 \\ 1 \\ 1 \\ -1 \end{bmatrix}
$$

All that needs to happen to change the input into the goal is that the $\ket{11}$ basis state needs to have its sign flipped.

Remember that the Pauli Z gate flips signs in the single qubit case, and that $CZ$ is the 2-qubit version of this gate. And indeed, the effect of the $CZ$ gate is exactly the transformation you're looking for here.

@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_2_solution_a",
"codePath": "./SolutionA.qs"
})
Alternatively, you can express this gate using the intrinsic $Z$ gate and its controlled variant using the Controlled functor:

@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_2_solution_b",
"codePath": "./SolutionB.qs"
})
