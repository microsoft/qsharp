As we've seen in the 'Superposition of all basis vectors on two qubits' task, to prepare a superposition of all basis vectors on 2 qubits we need to apply a Hadamard gate to each of the qubits.

It seems that the solution for the general case might be to apply a Hadamard gate to every qubit as well. Let's check the first few examples:

$$\begin{align*}
   H|0\rangle &= \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big) \\
   H|0\rangle \otimes H|0\rangle &= \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) \otimes \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) \\ 
               &= \frac{1}{\sqrt{2^2}}\big(|00\rangle + |01\rangle+ |10\rangle+ |11\rangle\big) \\
   H|0\rangle \otimes H|0\rangle \otimes H|0\rangle &= \frac{1}{\sqrt{2^2}}\big(|00\rangle + |01\rangle+ |10\rangle+ |11\rangle\big) \otimes \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big) \\
               &= \frac{1}{\sqrt{2^3}}\big(|000\rangle + |001\rangle + |010\rangle+ |100\rangle+ |110\rangle + |101\rangle+ |011\rangle+ |111\rangle\big) \\
    \underset{N}{\underbrace{H|0\rangle \otimes \dots \otimes H|0\rangle}} 
               &= \frac{1}{\sqrt{2^{N-1}}}  \big( |\underset{N-1}{\underbrace{0 \cdots 0}}\rangle + \cdots + |\underset{N-1}{\underbrace{1 \cdots 1}}\rangle \big) \otimes \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big) =  \\
               &= \frac{1}{\sqrt{2^N}} \big( |\underset{N}{\underbrace{0 \cdots 0}}\rangle + \cdots + |\underset{N}{\underbrace{1 \cdots 1}}\rangle \big) 
\end{align*}$$

Thus, the solution requires us to iterate over the qubit array and to apply the Hadamard gate to each element.

@[solution]({
    "id": "superposition__all_basis_vectors_solution",
    "codePath": "./Solution.qs"
})
