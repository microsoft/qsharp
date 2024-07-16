We begin in the same state as the previous exercise:
$$ \begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\ 0 \end{bmatrix} \otimes \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \ket{0} \otimes \ket{0}$$

The goal state can be separated as follows:
$$ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ -1 \\ 0 \\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\ 0 \end{bmatrix} \otimes \frac{1}{\sqrt2}\begin{bmatrix} 1 \\ -1 \end{bmatrix} = \ket{0} \otimes \frac{1}{\sqrt2}\big(\ket{0} - \ket{1}\big)$$

This means that the first qubit is already in the state we want it to be, but the second qubit needs to be transformed from the $ \begin{bmatrix} 1 \\ 0 \end{bmatrix} $ into $ \frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\ -1\end{bmatrix}$ state.

First, we apply the **X** gate to the second qubit; this performs the following transformation:
$$ X \ket{0} = \begin{bmatrix}0 & 1 \\ 1 & 0 \end{bmatrix} \cdot \begin{bmatrix}1 \\ 0 \end{bmatrix} = \begin{bmatrix} 0 \\ 1 \end{bmatrix} = \ket{1}  $$

Second, we apply the **H** gate to the second qubit; this transforms its state into the desired one:
$$ H\ket{1} = \frac{1}{\sqrt2}\begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix} \cdot \begin{bmatrix} 0 \\ 1 \end{bmatrix} = \frac{1}{\sqrt2}\begin{bmatrix} 1 \\ -1 \end{bmatrix}$$

@[solution]({
"id": "multi_qubit_systems__prepare_superposition_solution",
"codePath": "Solution.qs"
})
