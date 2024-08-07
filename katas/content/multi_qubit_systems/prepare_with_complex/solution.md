The start state is the same as the previous exercises:
$$ \begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\ 0 \end{bmatrix} \otimes \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \ket{0} \otimes \ket{0} $$

The goal state, factored as a tensor product, looks like this (remember that $e^{3i\pi/4} = e^{i\pi/4} e^{i\pi/2}$):

$$
\frac{1}{2}\begin{bmatrix} 1 \\ e^{i\pi/4} \\ e^{i\pi/2} \\ e^{3i\pi/4} \end{bmatrix} =
\frac{1}{\sqrt2} \begin{bmatrix} 1 \\ e^{i\pi/2} \end{bmatrix} \otimes \frac{1}{\sqrt2}\begin{bmatrix} 1 \\ e^{i\pi/4} \end{bmatrix} =
\frac{1}{\sqrt2}\big(\ket{0} + e^{i\pi/2}\ket{1}\big) \otimes \frac{1}{\sqrt2}\big(\ket{0} + e^{i\pi/4}\ket{1}\big) $$

You will again need to adjust the states of both qubits independently.

For the first qubit, you'll start by applying the $H$ gate, getting the state $\frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 1 \end{bmatrix}$, as you've seen in the previous task. Afterwards you'll apply the $S$ gate with the following result:

$$ \begin{bmatrix} 1 & 0 \\ 0 & i \end{bmatrix} \cdot \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ i \end{bmatrix}$$

If you recall that $i = e^{i\pi/2}$, you can write the final state of the first qubit as:
$$ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ e^{i\pi/2} \end{bmatrix} $$

For the second qubit, you'll apply the $H$ gate, followed by the $T$ gate, with the following result:
$$ \begin{bmatrix} 1 & 0 \\ 0 & e^{i\pi/4} \end{bmatrix} \cdot \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 1 \end{bmatrix} = \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ e^{i\pi/4} \end{bmatrix} $$

@[solution]({
"id": "multi_qubit_systems__prepare_with_complex_solution",
"codePath": "Solution.qs"
})
