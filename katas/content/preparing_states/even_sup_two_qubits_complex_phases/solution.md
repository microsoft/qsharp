You'll start approaching the problem from the desired end result. Let’s see if you can factor any expressions out of $\big(\ket{00} + i\ket{01} - \ket{10} - i\ket{11}\big)$:

$$
\ket{00} + i\ket{01} - \ket{10} - i\ket{11}
= \ket{00} + \big(\ket{0} - \ket{1}\big) i\ket{1} - \ket{10} =
$$
$$
= \big(\ket{0} - \ket{1}\big) \ket{0} + \big(\ket{0} - \ket{1}\big) i\ket{1}
= \big(\ket{0} - \ket{1}\big) \otimes \big(\ket{0} + i\ket{1}\big)
$$

The fact that you were able to factor out the state into a tensor product of two terms means the state is separable.

This is looking promising. Now let’s try to approach the problem from the other end, that is, from the starting state of $\ket{00}$. 
As you've seen in the previous task, applying a Hadamard operation to each $\ket{0}$ gets you closer to the factored-out expression:

$$
H\ket{0} \otimes H\ket{0} = \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) \otimes \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big)
=\frac{1}{2} \big(\ket{0} + \ket{1}\big) \otimes \big(\ket{0} + \ket{1}\big) 
$$

If you compare these two equations (while ignoring the $\frac{1}{2}$ term in the second equation), you end up with the following transformations that you need to perform on the individual qubits:

$$
\ket{0} + \ket{1} \overset{???}\rightarrow \ket{0} - \ket{1}
$$

$$
\ket{0} + \ket{1} \overset{???}\rightarrow \ket{0} + i\ket{1}
$$


Next, let's take a look at the basic gates, in particular the Pauli Z gate:

$$Z = \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$$

If it's applied to the state $\frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big)$, it'll leave the basis state $\ket{0}$ unchanged and will map $\ket{1}$ to $-\ket{1}$. Thus, 

$$Z\frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) = \frac{1}{\sqrt2} \big(\ket{0} - \ket{1}\big)$$

So the $Z$ gate is the answer to the question of how to do the first transformation. 

Looking for another gate to address the second transformation, you find the $S$ gate:

$$S = \begin{bmatrix} 1 & 0 \\ 0 & i \end{bmatrix}$$ 

If it's applied to the state $\frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big)$, it'll leave the basis state $\ket{0}$ unchanged and will map $\ket{1}$ to $i\ket{1}$. Thus, 

$$S\frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) = \frac{1}{\sqrt2} \big(\ket{0} + i\ket{1}\big)$$

So the $S$ gate now answers the question of how to do the second transformation.

To summarize, the state you need to prepare can be represented as follows:
$$ZH\ket{0} \otimes SH\ket{0}$$

Remember that in Q# the gates have to be applied in reverse order compared to the mathematical notation - the gate closest to the ket symbol is applied first.

@[solution]({
    "id": "preparing_states__even_sup_two_qubits_complex_phases_solution",
    "codePath": "./Solution.qs"
})
