You can see that the even superposition of two states from the first and the third tasks in this lesson will give the desired result. Indeed,

$$\frac1{\sqrt{2^{n-1}}} \big(\ket{0} + \ket{2} + ... + \ket{2^n-2}\big) =$$
$$=\frac{1}{\sqrt{2}} \bigg( \frac1{\sqrt{2^n}} \big(\ket{0} - \ket{1} + \ket{2} - \ket{3} + ... - \ket{2^n-1}\big) + \frac1{\sqrt{2^n}} \big(\ket{0} + \ket{1} + \ket{2} + \ket{3} + ... + \ket{2^n-1}\big)\bigg)$$

Now, you can use the fact that QFT is a linear transformation.
The two periodic states from earlier tasks are created by applying the QFT to the states $\ket{100...0}$ and $\ket{000...0}$ state. 
To prepare an equal superposition of these two states, you can apply a Hadamard gate to the first qubit in the register:

$$ \frac1{\sqrt2} \big(\ket{0} + \ket{1}\big) \otimes \ket{00...0} $$

Then you apply QFT to the register: 

$$ QFT\big(\frac1{\sqrt2} (\ket{0} + \ket{1}) \otimes \ket{00...0}\big) = \frac1{\sqrt2} \big( QFT\ket{000...0} + QFT\ket{100...0}\big)$$

This results in the desired end state. 

@[solution]({
    "id": "qft__all_even_vectors_solution",
    "codePath": "./Solution.qs"
})
