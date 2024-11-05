You know that the Hadamard gate maps the basis state $\ket{0}$ to $\frac{1}{\sqrt2}(\ket{0} + \ket{1})$, so it's a logical starting point for solving this problem. 

Next, you see that the final state has a $\frac{1}{2}$ term hinting that you might be applying two operations involving a $\frac{1}{\sqrt{2}}$ term. 

Now, how do you get the $\ket{00} + \ket{01} + \ket{10} + \ket{11}$ expression? Let's see what does multiplying the expression $\ket{0} + \ket{1}$ by itself look like:

$$\big(\ket{0} + \ket{1}\big) \otimes \big(\ket{0} + \ket{1}\big) = \ket{0}\ket{0} + \ket{0}\ket{1} + \ket{1}\ket{0} + \ket{1}\ket{1}$$

Thus, applying the Hadamard gate to each qubit in isolation will deliver the desired final result:

$$H\ket{0} \otimes H\ket{0} = \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) \otimes \frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big)
= \frac{1}{2} (\ket{00} + \ket{01} + \ket{10} + \ket{11})$$

Q# arrays are similar to arrays in other languages: you can access the $i$-th element of the array `qs` as `qs[i]` (indices are 0-based).

@[solution]({
    "id": "preparing_states__even_sup_two_qubits_solution",
    "codePath": "./Solution.qs"
})
