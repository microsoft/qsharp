As you've seen in the 'Superposition of all basis vectors on two qubits' task, to prepare a superposition of all basis vectors on 2 qubits you need to apply a Hadamard gate to each of the qubits.

It seems that the solution for the general case might be to apply a Hadamard gate to every qubit as well. Let's check the first few examples:

$$\begin{align*}
   H\ket{0} &= \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big) \\
   H\ket{0} \otimes H\ket{0} &= \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) \otimes \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) \\ 
               &= \frac{1}{\sqrt{2^2}}\big(\ket{00} + \ket{01}+ \ket{10}+ \ket{11}\big) \\
   H\ket{0} \otimes H\ket{0} \otimes H\ket{0} &= \frac{1}{\sqrt{2^2}}\big(\ket{00} + \ket{01}+ \ket{10}+ \ket{11}\big) \otimes \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big) \\
               &= \frac{1}{\sqrt{2^3}}\big(\ket{000} + \ket{001} + \ket{010}+ \ket{100}+ \ket{110} + \ket{101}+ \ket{011}+ \ket{111}\big) \\
    \underset{N}{\underbrace{H\ket{0} \otimes \dots \otimes H\ket{0}}} 
               &= \frac{1}{\sqrt{2^{N-1}}}  \big( \ket{\underset{N-1}{\underbrace{0 \cdots 0}}} + \cdots + \ket{\underset{N-1}{\underbrace{1 \cdots 1}}} \big) \otimes \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big) =  \\
               &= \frac{1}{\sqrt{2^N}} \big( \ket{\underset{N}{\underbrace{0 \cdots 0}}} + \cdots + \ket{\underset{N}{\underbrace{1 \cdots 1}}} \big) 
\end{align*}$$

Thus, the solution requires you to iterate over the qubit array and to apply the Hadamard gate to each element.

@[solution]({
    "id": "preparing_states__all_basis_vectors_solution",
    "codePath": "./Solution.qs"
})
