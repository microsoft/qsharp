Alice and Bob, independently from each other, each hold an entangled qubit pair in the state $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$. They hand off one part of their pair to Charlie.

Charlie can now teleport the state of Alice's qubit he holds onto Bob's remaining qubit, thus teleporting the entanglement. Just like in `standard` teleportation, Bob still needs to apply the reconstruction steps - based on Charlie's measurement results - to the other qubit in his possession.

After this procedure the state $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$ now spans across Alice's and Bob's qubits which they didn't send to Charlie. They are now maximally entangled, even though they never interacted in the first place!

**Goal:** 
A tuple of two operations.

The first operation is Charlie's part of the protocol. It will take two qubits as input (the ones Alice and Bob sent to Charlie), and produce a message, encoded as bool, that will be sent to Bob.

The second operation is Bob's part of the protocol. It will take the qubit that remained in Bob's possession and Charlie's encoded bools as input, and use the boolean values to adjust the state of Bob's qubit, so that Alice's and Bob's qubits end up in the state $\ket{\Phi^{+}}$.

**Note:**
You will likely need to create two separate helper operations that implement the two parts of the protocol, and return them, rather than implementing the solution in the body of this operation.