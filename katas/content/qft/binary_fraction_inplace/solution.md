First, let's recall the first task of the kata: a Hadamard gate applied to a single qubit,  $H\ket{j_1}$, will give either
$\frac1{\sqrt2}(\ket{0} + \ket{1})$ or $\frac{1}{\sqrt{2}}(\ket{0} - \ket{1})$ depending on the state of $\ket{j_1}$. This operation can also be written as
$$H\ket{j_1} = \frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j_1}{2}}\ket{1} \big)= \frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_1}\ket{1} \big)$$

So, if the starting register state is $\ket{j_1 j_2 ... j_n}$, applying a Hadamard gate to the first qubit will result in:

$$\big(H_1\otimes I_{n-1} \big) \big(\ket{j_1} \otimes \ket{j_2 ... j_n} \big)= \frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_1}\ket{1} \big) \otimes \ket{j_2 ... j_n} $$

After this, we can repeat the loop we used in the previous task for qubits $\ket{j_2 ... j_n}$ with the first qubit as the target to adjust the remaining phase terms via the controlled $\textrm{R1Frac}$ gate.

@[solution]({
"id": "qft__binary_fraction_inplace_solution",
"codePath": "./Solution.qs"
})
