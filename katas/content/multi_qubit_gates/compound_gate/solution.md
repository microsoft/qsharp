One way to represent a multi-qubit transformation is to use the tensor product of gates acting on subsets of qubits. For example, if you have 2 qubits, applying the $Z$ gate on the first qubit and the $X$ gate on the second qubit will create this matrix:

$$
Z \otimes X =
\begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix} \otimes \begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix} =
\begin{bmatrix} 0 & 1 & 0 & 0 \\ 1 & 0 & 0 & 0 \\ 0 & 0 & 0 & -1 \\ 0 & 0 & -1 & 0 \end{bmatrix}
$$

With this in mind, let's see how to reverse engineer the target matrix above to find the 3 gates which, acting on individual qubits, together form the target transformation.

Start by noticing that the top right and bottom left quadrants of the target matrix are filled with $0$'s, and the bottom right quadrant equals to the top left one, multiplied by $i$. This hints at applying the $S$ gate to the first qubit:

$$
Q =
\begin{bmatrix} 1 & 0 \\ 0 & i \end{bmatrix} \otimes
\begin{bmatrix}
    0 & -i & 0 & 0 \\ 
    i & 0 & 0 & 0  \\ 
    0 & 0 & 0 & -i \\ 
    0 & 0 & i & 0 
\end{bmatrix} =
\begin{bmatrix}
    0 & -i & 0 & 0 & 0 & 0 & 0 & 0 \\ 
    i & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\ 
    0 & 0 & 0 & -i & 0 & 0 & 0 & 0 \\ 
    0 & 0 & i & 0 & 0 & 0 & 0 & 0 \\ 
    0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\ 
    0 & 0 & 0 & 0 & -1 & 0 & 0 & 0 \\ 
    0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\ 
    0 & 0 & 0 & 0 & 0 & 0 & -1 & 0
\end{bmatrix}
$$

Now the $4 \times 4$ matrix has all $0$s in the top right and bottom left quadrants, and the bottom right quadrant equals to the top left one. This means the second qubit has the $I$ gate applied to it, and the third qubit - the $Y$ gate:

$$
Q =
\begin{bmatrix} 1 & 0 \\ 0 & i \end{bmatrix} \otimes \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix} \otimes
\begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix} = S \otimes I \otimes Y
$$

@[solution]({
    "id": "multi_qubit_gates__compound_gate_solution",
    "codePath": "./Solution.qs"
})
