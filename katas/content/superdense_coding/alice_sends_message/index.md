**Inputs**:

1. Alice's part of the entangled pair of qubits qAlice.
2. Two classical bits

**Goal**:  
Encode the message (two classical bits) in the state of Alice's qubit.

> `message` is a tuple of 2 `Bool` variables to represent `bit1` and `bit2` respectively.

<details>
  <summary><b>Need a hint? Click here</b></summary>
Manipulate Alice's half of the entangled pair to change the joint state of the two qubits to one of the following four states based on the value of message:

- `(0, 0)`: $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} (\ket{00} + \ket{11})$
- `(0, 1)`: $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} (\ket{01} + \ket{10})$
- `(1, 0)`: $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} (\ket{00} - \ket{11})$
- `(1, 1)`: $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} (\ket{01} - \ket{10})$

</details>
