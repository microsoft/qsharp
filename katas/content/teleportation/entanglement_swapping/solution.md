The exercise demonstrate the capability of sharing a bell pair between Alice and Bob without any direct communication. This requires a third party Charlie who both Alice and Bob trust.

The implementation can be divided into following operations:

- Individually, Alice and Bob prepare a Bell state each, two qubits in the $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$ state.
- Charlie now receives one qubit from Alice `qAlice1` and other from Bob `qBob1`.
- Now, Charlie treats Alice's qubit `qAlice1` as the message qubit and teleports its state to Bob. Using the `SendMessage` operation implemented in the earlier exercise, Charlie gets two `Bool` values and encodes them as one `Int` - the classical message. Based on Charlie's measurement results, the remaining two qubits, `qAlice2` and `qBob2` can be in one of the four Bell states.
- Finally, using the `ReconstructMessage` operation implemented in the earlier exercise, Bob can perform correction on the `qBob2` qubit to convert the state of the qubit pair of `qAlice2` and `qBob2` to $\ket{\Phi^{+}}$.

In this exercise you only need to implement the last two steps.

@[solution]({
    "id": "teleportation__entanglement_swapping_solution",
    "codePath": "./Solution.qs"
})