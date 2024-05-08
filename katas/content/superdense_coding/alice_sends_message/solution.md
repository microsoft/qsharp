Recall that we learnt how to prepare all Bell states in "Preparing Quantum States" kata. This is slightly advanced version of that task demonstrating how operations applied to one qubit of a Bell state allow us to transform it into any other Bell state.

Superdense coding protocol uses the below Bell state we prepared in the previous task:

$$\frac{1}{\sqrt{2}} \big(\ket{00} + \ket{11} \big)$$

We can transform it into every other Bell state according to the value of `message`:

- `(0, 0)`: $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11})$
- `(0, 1)`: $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10})$
- `(1, 0)`: $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11})$
- `(1, 1)`: $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10})$

Here is how we can perform this transformation:

- If `bits == (0, 0)`, we do nothing - the prepared state is already $\ket{\Phi^{+}}$.

- If `bits ==  (0, 1)`, we need to change the second qubit in both $\ket{00}$ and $\ket{11}$ terms. Observe that applying an $X$ gate to Alice's qubit does exactly that:
  $$(X \otimes I) \ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{10} + \ket{01})$$

- If `bits == (1, 0)`, we need to add a relative phase of $-1$ to the $\ket{11}$ term. Observe that applying $Z$ gate to Alice's qubit does exactly that:
  $$(Z \otimes I) \ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11})$$

- If `bits = (1, 1)`, we use the same logic to realize that we need to apply both the $Z$ and $X$ corrections to get $\ket{\Psi^{-}}$ state.
  $$ (Z \otimes I) \cdot (X \otimes I) \ket{\Psi^{+}} = (Z \otimes I) \frac{1}{\sqrt{2}} (\ket{10} + \ket{01}) = \frac{1}{\sqrt{2}} (-\ket{10} + \ket{01}) = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10}) $$

The final sequence of steps is as follows:

1. Apply the $X$ gate to Alice's qubit if `bit2 == 1`.
2. Apply the $Z$ gate to Alice's qubit if `bit1 == 1`.

@[solution]({
    "id": "superdense_coding__alice_sends_message_solution",
    "codePath": "./Solution.qs"
})
