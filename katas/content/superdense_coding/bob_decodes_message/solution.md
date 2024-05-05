Recall that Alice encoded qubits as follows:

- `(0, 0)`: $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11})$
- `(0, 1)`: $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10})$
- `(1, 0)`: $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11})$
- `(1, 1)`: $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10})$

To get our state back, we can undo the entanglement by applying $CNOT$ and $H$ gate.
Notice that it's important to keep the order right. The qubits that are subject to the Hadamard transform and the $CNOT$ gate in the preparation of the pair have to match the operations below, or the order of the data bits will get flipped.

Notice that [`Adjoint`](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/functorapplication#adjoint-functor) functor in Q# does exactly that.
  
What is the outcome of this transformation, assuming each of the possible quantum states after the encoding step?

- $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11}) --> \ket{00}$
- $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10}) --> \ket{01}$
- $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11}) --> \ket{10}$
- $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10}) --> \ket{11}$

Hence, we can retrieve the encoded bits just by measuring the bits.

@[solution]({
    "id": "superdense_coding__bob_decodes_message_solution",
    "codePath": "./Solution.qs"
})
