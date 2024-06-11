We can represent the required state as follows:

$$\frac{1}{2}(\ket{000}+\ket{011}+\ket{101}+\ket{110}) = \frac1{\sqrt2} \left( \ket{0} \otimes \frac1{\sqrt2}(\ket{00} + \ket{11}) + \ket{1} \otimes \frac1{\sqrt2}(\ket{01} + \ket{10}) \right)$$
 
In other words, depending on the state of qubit `qAlice`, the other two qubits are prepared in one of the Bell states.

- Starting with $\ket{000}$, the first step is to prepare `qBob` and `qCharlie` in the Bell state $\frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$. This can be done using Hadamard and $CNOT$ operations, similar to the `Entangle` exercise. This gives the resultant state as $\frac{1}{\sqrt{2}}(\ket{000} + \ket{011})$.
- After this, applying a Hadamard gate to `qAlice` qubit gives us the state as $\frac{1}{2}(\ket{000} + \ket{011} + \ket{100} + \ket{111})$.
- Finally, we need to adjust the basis states for which `qAlice` is in the $\ket{1}$ state, flipping the Bell state we have on the other two qubits to a different one. We can do this using a $CNOT$ gate with `qAlice` as control and either `qBob` or `qCharlie` as target.

@[solution]({
    "id": "teleportation__entangled_trio_solution",
    "codePath": "./Solution.qs"
})