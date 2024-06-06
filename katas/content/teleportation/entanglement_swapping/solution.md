The exercise demonstrate the capability of sharing a bell pair between Alice and Bob without any direct communication. This requires a third party Charlie who both Alice and Bob trust.

The implementation can be divided into following operations:

- Individually, Alice and Bob are required to prepare a bell state each, both in $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$ state.
- Charlie now receives one qubit from Alice `qAlice1` and other from Bob `qBob1`.
- Using the `SendMessage` operation implemented in the earlier exercise, Charlie prepares the classical message as Int. Based on this classical result, remaining two qubits, `qAlice2` and `qBob2` can be in one of the four bell states.
- Finally, using the `ReconstructMessage` operation implemented in the earlier exercise, Bob can perform correction on the `qBob2` qubit to bring back the bell pair of `qAlice2` and `qBob2` to $\ket{\Phi^{+}}$.

Since the exercise only excepts functions, implementation of ast two steps is only required.

@[solution]({
    "id": "teleportation__entanglement_swapping_solution",
    "codePath": "./Solution.qs"
})