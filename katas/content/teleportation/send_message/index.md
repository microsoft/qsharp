Entangle the message qubit with Alice's qubit and extract two classical bits to be sent to Bob.

**Inputs:** 
1. Alice's part of the entangled pair of qubits `qAlice`.
2. The message qubit `qMessage`.

**Output:** 
Two classical bits Alice will send to Bob via classical channel as a tuple of Boolean values. The first bit in the tuple should hold the result of measurement of the message qubit, the second bit - the result of measurement of Alice's qubit. Represent measurement result `One` as `true` and `Zero` as `false`. The state of the qubits in the end of the operation doesn't matter.