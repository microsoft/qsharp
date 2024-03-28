We know that the Hadamard gate maps the basis state $|0\rangle$ to $\frac{1}{\sqrt2}(|0\rangle + |1\rangle)$, so it is a logical starting point for solving this problem. 

Next, we see that the final state has a $\frac{1}{2}$ term hinting that we might be applying two operations involving a $\frac{1}{\sqrt{2}}$ term. 

Now, how do we get the $|00\rangle + |01\rangle + |10\rangle + |11\rangle$ expression? Let's see what does multiplying the expression $|0\rangle + |1\rangle$ by itself look like:

$$\big(|0\rangle + |1\rangle\big) \otimes \big(|0\rangle + |1\rangle\big) = |0\rangle|0\rangle + |0\rangle|1\rangle + |1\rangle|0\rangle + |1\rangle|1\rangle$$

Thus, applying the Hadamard gate to each qubit in isolation will deliver the desired final result:

$$H|0\rangle \otimes H|0\rangle = \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) \otimes \frac{1}{\sqrt2}\big(|0\rangle + |1\rangle\big)
= \frac{1}{2} (|00\rangle + |01\rangle + |10\rangle + |11\rangle)$$

Q# arrays are similar to arrays in other languages: you can access the $i$-th element of the array `qs` as `qs[i]` (indices are 0-based).

@[solution]({
    "id": "superposition__even_sup_two_qubits_solution",
    "codePath": "./Solution.qs"
})
