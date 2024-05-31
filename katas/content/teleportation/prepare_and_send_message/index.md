Given a Pauli basis along with a state `true` as `One` or `false` as `Zero`, prepare a message qubit, entangle it with Alice's qubit, and extract two classical bits to be sent to Bob.

**Inputs:** 
1. Alice's part of the entangled pair of qubits `qAlice`.
2. A PauliX, PauliY, or PauliZ basis in which the message qubit should be prepared.
3. A Bool indicating the eigenstate in which the message qubit should be prepared.

**Outputs:** 
Two classical bits Alice will send to Bob via classical channel as a tuple of Bool values. The first bit in the tuple should hold the result of measurement of the message qubit and the second bit, the result of measurement of Alice's qubit. Represent measurement result `One` as `true` and `Zero` as `false`. The state of the qubit `qAlice` in the end of the operation doesn't matter.