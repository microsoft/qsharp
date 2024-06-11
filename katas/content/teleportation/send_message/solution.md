This step of the protocol is equivalent to performing a measurement in Bell basis. Let's see how the steps Alice performs affect the joint state of the three qubits involved in the protocol.

Initially, the message qubit is in the state $\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$, and Alice's and Bob's qubits are in the Bell state $\ket{\Phi^{+}} = \frac1{\sqrt2}(\ket{00} + \ket{11})$. We'll track the evolution of the state by writing out the states of the qubits in that order: the message qubit, Alice's qubit, Bob's qubit.

$$\ket{\psi} \ket{\Phi^{+}} = (\alpha \ket{0} + \beta \ket{1}) \frac1{\sqrt2}(\ket{00} + \ket{11}) = $$
$$= \alpha \ket{0} \frac1{\sqrt2}(\ket{00} + \ket{11}) + \beta \ket{1} \frac1{\sqrt2}(\ket{00} + \ket{11})$$

Next, we apply the $CNOT$ gate with `qMessage` as control and `qAlice` as target. The state becomes

$$\alpha \ket{0} \frac1{\sqrt2}(\ket{00} + \ket{11}) + \beta \ket{1} \frac1{\sqrt2}(\ket{\textbf{1}0} + \ket{\textbf{0}1})$$

After this, we apply the Hadamard gate on `qMessage`:

$$\alpha \mathbf{\frac1{\sqrt2}(\ket{0} + \ket{1})} \frac1{\sqrt2}(\ket{00} + \ket{11}) + \beta \mathbf{\frac1{\sqrt2}(\ket{0} - \ket{1})} \frac1{\sqrt2}(\ket{10} + \ket{01})$$

Finally, we measure both qubits and return the measurement results as a tuple of Boolean values.
To make it easier to see the effects of this step on the state, let's regroup the terms based on the state of the first two qubits:

$$\frac12 \left( \mathbf{\ket{00}} (\alpha \ket{0} + \beta \ket{1}) \right) +
  \frac12 \left( \mathbf{\ket{01}} (\alpha \ket{1} + \beta \ket{0}) \right) +$$
$$+ \frac12 \left( \mathbf{\ket{10}} (\alpha \ket{0} - \beta \ket{1}) \right) +
    \frac12 \left( \mathbf{\ket{11}} (\alpha \ket{1} - \beta \ket{0}) \right)$$

Now, when Alice measures the first two qubits, Bob's qubit ends up in a state with amplitudes that depend on the amplitudes of the original message qubit.

@[solution]({
    "id": "teleportation__send_the_message_solution",
    "codePath": "./Solution.qs"
})