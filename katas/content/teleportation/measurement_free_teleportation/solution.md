Let's see how we can perform teleportation without using measurements.

We start similarly to the `SendMessage` operation in the earlier exercise, by applying a $CNOT$ gate followed by a Hadamard gate. After this, the state of the three-qubit system will be the same as it is in the regular teleportation protocol:

$$\frac12 \left( \ket{00} (\alpha \ket{0} + \beta \ket{1}) + \ket{01} (\alpha \ket{1} + \beta \ket{0}) + \ket{10} (\alpha \ket{0} - \beta \ket{1}) + \ket{11} (\alpha \ket{1} - \beta \ket{0}) \right)$$

Now, in the standard protocol we would measure the first two qubits to collapse the last one into one of superpositions involving coefficients $\alpha$ and $\beta$. Instead, let's see how we can adjust the state of the Bob's qubit to become $\alpha \ket{0} + \beta \ket{1}$ for each of the possible values of the first two qubits.

To do this, we need to replace each measurement followed by a classically conditioned gate with a controlled gate:

- The $Z$ gate applied based on the measurement result of `qMessage` becomes a Controlled $Z$ with `qMessage` as control and `qBob` as target.
- The $X$ gate applied based on the measurement result of `qAlice` becomes a Controlled $X$ with `qAlice` as control and `qBob` as target.

You can check that these two gates convert Bob's qubit into the required state, and leave the first two qubits in an equal superposition of all basis states.

@[solution]({
    "id": "teleportation__measurement_free_teleportation_solution",
    "codePath": "./Solution.qs"
})