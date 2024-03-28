As we've seen in the previous task, the Hadamard gate maps the basis state $|0\rangle$ to $\frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big)$ and $|1\rangle$ to $\frac{1}{\sqrt2}\big(|0\rangle - |1\rangle\big)$. 
If our qubit was already in the $|1\rangle$ state, we would simply apply the Hadamard gate to prepare the required $|-\rangle$ state. 
Fortunately, there is another operation we can use to change the state $|0\rangle$ to $|1\rangle$, namely the X gate:

$$X = \begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix}$$

This gate transforms $|0\rangle \longmapsto |1\rangle$ and $|1\rangle \longmapsto |0\rangle$.
X is another one of the built-in gates in Q# from the `Microsoft.Quantum.Intrinsic` namespace.

Thus, our solution should apply the X gate to our qubit, followed by the Hadamard gate.

@[solution]({
    "id": "superposition__minus_state_solution",
    "codePath": "./Solution.qs"
})
