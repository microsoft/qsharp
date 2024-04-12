We will start approaching the problem from the desired end result. Let’s see if we can factor any expressions out of $\big(|00\rangle + i|01\rangle - |10\rangle - i|11\rangle\big)$:

$$
|00\rangle + i|01\rangle - |10\rangle - i|11\rangle
= |00\rangle + \big(|0\rangle - |1\rangle\big) i|1\rangle - |10\rangle =
$$
$$
= \big(|0\rangle - |1\rangle\big) |0\rangle + \big(|0\rangle - |1\rangle\big) i|1\rangle
= \big(|0\rangle - |1\rangle\big) \otimes \big(|0\rangle + i|1\rangle\big)
\label{5.1} \tag{5.1}
$$

The fact that we were able to factor out the state into a tensor product of two terms means the state is separable.

This is looking promising.  Now let’s try to approach the problem from the other end, i.e. from the starting state of $|00\rangle$. 
As we've seen in the previous task, applying a Hadamard operation to each $|0\rangle$ gets us closer to the factored-out expression:

$$
H|0\rangle \otimes H|0\rangle = \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) \otimes \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big)
=\frac{1}{2} \big(|0\rangle + |1\rangle\big) \otimes \big(|0\rangle + |1\rangle\big) 
\label{5.2} \tag{5.2}
$$

If we compare equations 5.1 and 5.2 (while ignoring the $\frac{1}{2}$ term in equation 5.2), we end up with the following transformations that we need to perform on the individual qubits:

$$
|0\rangle + |1\rangle \overset{???}\rightarrow |0\rangle - |1\rangle
\label{5.3} \tag{5.3}
$$

$$
|0\rangle + |1\rangle \overset{???}\rightarrow |0\rangle + i|1\rangle
\label{5.4} \tag{5.4}
$$


Next lets take a look at our basic gates, in particular the Pauli Z gate:

$$Z = \begin{bmatrix} 1 & 0 \\\ 0 & -1 \end{bmatrix}$$

If it is applied to the state $\frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big)$, it will leave the basis state $|0\rangle$ unchanged and will map $|1\rangle$ to $-|1\rangle$. Thus, 

$$Z\frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) = \frac{1}{\sqrt2} \big(|0\rangle - |1\rangle\big)$$

So the Z gate is the answers to the question of how to do the conversion 5.3. 

Looking for another gate to address the conversion 5.4, we find the S gate:

$$S = \begin{bmatrix} 1 & 0 \\\ 0 & i \end{bmatrix}$$ 

If it is applied to the state $\frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big)$, it will leave the basis state $|0\rangle$ unchanged and will map $|1\rangle$ to $i|1\rangle$. Thus, 

$$S\frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) = \frac{1}{\sqrt2} \big(|0\rangle + i|1\rangle\big)$$

So the S gate now answers the question of how to do the conversion 5.4.

To summarize, the state we need to prepare can be represented as follows:
$$ZH|0\rangle \otimes SH|0\rangle$$

Remember that in Q# the gates have to be applied in reverse order compared to the mathematical notation - the gate closest to the ket symbol is applied first.

@[solution]({
    "id": "superposition__even_sup_two_qubits_complex_phases_solution",
    "codePath": "./Solution.qs"
})
