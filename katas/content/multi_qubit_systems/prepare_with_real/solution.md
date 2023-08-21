Again, to start we will represent the goal state as a tensor product of single-qubit states; this gives us the following representation:

$$ \frac{1}{2}\big(|00\rangle - |01\rangle + |10\rangle - |11\rangle\big) = \frac{1}{2}\begin{bmatrix} 1 \\\ -1 \\\ 1 \\\ -1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 1 \end{bmatrix} \otimes \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ -1 \end{bmatrix} = \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big) \otimes \frac{1}{\sqrt2}\big(|0\rangle - |1\rangle\big)  $$

This time we need to transform both the first and the second qubits. Let's start with the first qubit. Applying the **H** gate transforms its state as follows:

$$ H|0\rangle = \frac{1}{\sqrt2}\begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix} \cdot \begin{bmatrix} 1 \\\ 0 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 1 \end{bmatrix} = \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big)$$

For the second qubit we can use the same transformation we've seen in the "Prepare a Superposition of Two Basis States" exercise; this will give the desired end state.

@[solution]({
    "id": "prepare_with_real_solution",
    "codePath": "solution.qs"
})
