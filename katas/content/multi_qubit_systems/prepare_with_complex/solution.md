The start state is the same as the previous exercises:
$$ \begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\\ 0 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix} = |0\rangle \otimes |0\rangle $$

The goal state, factored as a tensor product, looks like this (remember that $e^{3i\pi/4} = e^{i\pi/4} e^{i\pi/2}$):

$$
\frac{1}{2}\begin{bmatrix} 1 \\\ e^{i\pi/2} \\\ e^{i\pi/4} \\\ e^{3i\pi/4} \end{bmatrix} =
\frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ e^{i\pi/4} \end{bmatrix} \otimes \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ e^{i\pi/2} \end{bmatrix} =
\frac{1}{\sqrt2}\big(|0\rangle + e^{i\pi/4}|1\rangle\big) \otimes \frac{1}{\sqrt2}\big(|0\rangle + e^{i\pi/2}|1\rangle\big) $$

We will again need to adjust the states of both qubits independently.

> Note that because these examples use a little-endian format, the "first" qubit (`qs[0]`) refers to the least-significant qubit whose target state is described by the right-hand operand of the tensor product, while the "second" qubit (`qs[1]`) refers to the most-significant qubit whose target state is described by the left-hand operand of the tensor product.

For the first qubit, we'll start by applying the **H** gate, getting the state $\frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 1 \end{bmatrix}$, as we've seen in the previous task. Afterwards we'll apply the **S** gate with the following result:

$$ \begin{bmatrix} 1 & 0 \\\ 0 & i \end{bmatrix} \cdot \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ i \end{bmatrix}$$

If we recall that $i = e^{i\pi/2}$, we can write the final state of the first qubit as:
$$ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ e^{i\pi/2} \end{bmatrix} $$

For the second qubit. we'll apply the **H** gate, followed by the **T** gate, with the following result:
$$ \begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/4} \end{bmatrix} \cdot \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ e^{i\pi/4} \end{bmatrix} $$

@[solution]({
"id": "multi_qubit_systems__prepare_with_complex_solution",
"codePath": "solution.qs"
})
