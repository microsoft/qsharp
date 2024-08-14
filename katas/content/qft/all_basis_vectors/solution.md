You can think of the required equal superposition of all basis states as the QFT of the state $\ket{0...0}$. Indeed, if $j_1 = j_2 = ... = j_n = 0$, $j = 0$, and you get the following state:

$$\frac1{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i \cdot \frac{jk}{2^{n}}} \ket{k} = \frac{1}{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{0} \ket{k} = \frac{1}{\sqrt{2^n}} \sum_{k=0}^{2^n-1} \ket{k}$$

This means that you can solve this task by simply applying the QFT to the given qubit register.

@[solution]({
    "id": "qft__all_basis_vectors_solution",
    "codePath": "./Solution.qs"
})
