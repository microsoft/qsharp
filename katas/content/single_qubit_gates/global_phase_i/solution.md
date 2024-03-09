We need to apply a gate which applies a global phase of $i$, i.e. $|\psi\rangle \rightarrow i|\psi\rangle$.
The matrix representation of such a gate is $\begin{bmatrix} i & 0 \\\ 0 & i \end{bmatrix} = i\begin{bmatrix} 1 & 0 \\\ 0 & 1 \end{bmatrix} = iI$.
Since we are restricted to the Pauli gates, we use the property that a product of any two distinct Pauli gates equals the third gate with a $+i$ or a $-i$ global phase, therefore the product of all three Pauli gates is $XYZ = iI$.
$$
\begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix}\begin{bmatrix} 0 & -i \\\ i & 0 \end{bmatrix}\begin{bmatrix} 1 & 0 \\\ 0 & -1 \end{bmatrix} = 
\begin{bmatrix} i & 0 \\\ 0 & i \end{bmatrix}
$$

> Remember the rightmost gates in mathematical notation are applied first in Q# code. Hence we first apply the $Z$ gate, followed by the $Y$ gate, and finally the $X$ gate.

@[solution]({
    "id": "single_qubit_gates__global_phase_i_solution",
    "codePath": "./Solution.qs"
})
