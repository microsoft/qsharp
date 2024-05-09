Recall that Alice encoded qubits as follows:

- `(0, 0)`: $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11})$
- `(0, 1)`: $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10})$
- `(1, 0)`: $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11})$
- `(1, 1)`: $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10})$

To read out the encoded message, Bob needs to figure out which of the four Bell states he has. We can map the Bell states to basis states by applying a $CNOT$ gate with the first qubit as control and the second qubit as target, followed by an $H$ gate on the first qubit. (Notice that this is exactly what [`Adjoint`](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/functorapplication#adjoint-functor) of the state preparation operation in the first task does.)
  
What is the outcome of this transformation, assuming each of the possible quantum states after the encoding step?

- $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11}) \rightarrow \ket{00}$
- $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10}) \rightarrow \ket{01}$
- $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11}) \rightarrow \ket{10}$
- $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10}) \rightarrow \ket{11}$

Hence, we can retrieve the encoded bits just by measuring the bits.

@[solution]({
    "id": "superdense_coding__bob_decodes_message_solution",
    "codePath": "./Solution.qs"
})
