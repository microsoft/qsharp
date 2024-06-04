Given a Pauli basis with a Result state, prepare a message qubit. 
This would mean preparing eigenstates of $\ket{0}$ or $\ket{1}$ for `PauliZ` basis, $\ket{+}$ or $\ket{-}$ for `PauliX` basis, and $\ket{i}$ or $\ket{-i}$ for `PauliY` basis. Then entangle this `qMessage` qubit with Alice's qubit, and extract two classical bits to be sent to Bob.

**Inputs:** 
1. Alice's part of the entangled pair of qubits `qAlice`.
2. A basis in which the message qubit should be prepared - `PauliX`, `PauliY`, or `PauliZ`.
3. A Result object indicating the qubit state.

**Outputs:** 
Two classical bits Alice will send to Bob via classical channel as a tuple of Bool values. The first bit in the tuple should hold the result of measurement of the message qubit and the second bit, the result of measurement of Alice's qubit. Represent measurement result `One` as `true` and `Zero` as `false`. The state of the qubit `qAlice` in the end of the operation doesn't matter.