We begin in the same state as the previous excercise:
$$ \begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\\ 0 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix} = |0\rangle \otimes |0\rangle$$

The goal state can be separated as follows:
$$ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 0 \\\ -1 \\\ 0 \end{bmatrix} = \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ -1 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix} = \frac{1}{\sqrt2}\big(|0\rangle - |1\rangle\big) \otimes |0\rangle$$

This means that the first qubit is already in the state we want it to be, but the second qubit needs to be transformed from the $ \begin{bmatrix} 1 \\\ 0 \end{bmatrix} $ into $ \frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ -1\end{bmatrix}$ state.

> Note that because these examples use a little-endian format, the "first" qubit (`qs[0]`) refers to the least-significant qubit whose target state is described by the right-hand operand of the tensor product, while the "second" qubit (`qs[1]`) refers to the most-significant qubit whose target state is described by the left-hand operand of the tensor product.

First, we apply the **X** gate to the second qubit; this performs the following transformation:
$$ X |0\rangle = \begin{bmatrix}0 & 1 \\\ 1 & 0 \end{bmatrix} \cdot \begin{bmatrix}1 \\\ 0 \end{bmatrix} = \begin{bmatrix} 0 \\\ 1 \end{bmatrix} = |1\rangle  $$

Second, we apply the **H** gate to the second qubit; this transforms its state into the desired one:
$$ H|1\rangle = \frac{1}{\sqrt2}\begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix} \cdot \begin{bmatrix} 0 \\\ 1 \end{bmatrix} = \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ -1 \end{bmatrix}$$

@[solution]({
"id": "multi_qubit_systems__prepare_superposition_solution",
"codePath": "solution.qs"
})
