Given a message represented as a Pauli basis and a Result value indicating a basis state of that basis, prepare a message qubit in that state. 
Result values `Zero` and `One` correspond to preparing states $\ket{0}$ or $\ket{1}$ for `PauliZ` basis, $\ket{+}$ or $\ket{-}$ for `PauliX` basis, and $\ket{i}$ or $\ket{-i}$ for `PauliY` basis. Then entangle this `qMessage` qubit with Alice's qubit, and extract two classical bits to be sent to Bob.

**Inputs:** 
1. Alice's part of the entangled pair of qubits `qAlice`.
2. A basis in which the message qubit should be prepared - `PauliX`, `PauliY`, or `PauliZ`.
3. A Result value indicating the qubit state.

**Outputs:** 
Two classical bits Alice will send to Bob via classical channel as a tuple of `Bool` values. Same as in the earlier task, the first bit in the tuple should hold the result of measurement of the message qubit and the second bit, the result of measurement of Alice's qubit. Represent measurement result `One` as `true` and `Zero` as `false`. The state of the qubit `qAlice` in the end of the operation doesn't matter.