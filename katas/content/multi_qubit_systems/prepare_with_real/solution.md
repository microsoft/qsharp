Again, to start we will represent the goal state as a tensor product of single-qubit states; this gives us the following representation:

$$ \frac{1}{2}\big(\ket{00} - \ket{01} + \ket{10} - \ket{11}\big) = \frac{1}{2}\begin{bmatrix} 1 \\ -1 \\ 1 \\ -1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 1 \end{bmatrix} \otimes \frac{1}{\sqrt2}\begin{bmatrix} 1 \\ -1 \end{bmatrix} = \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big) \otimes \frac{1}{\sqrt2}\big(\ket{0} - \ket{1}\big)  $$

This time we need to transform both the first and the second qubits. Let's start with the first qubit. Applying the **H** gate transforms its state as follows:

$$ H\ket{0} = \frac{1}{\sqrt2}\begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix} \cdot \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 1 \end{bmatrix} = \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big)$$

For the second qubit we can use the same transformation we've seen in the "Prepare a Superposition of Two Basis States" exercise; this will give the desired end state.

@[solution]({
    "id": "multi_qubit_systems__prepare_with_real_solution",
    "codePath": "Solution.qs"
})
