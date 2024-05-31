Transform Bob's qubit into the required state using the two classical bits received from Alice and measure it in the same basis in which she prepared the message.

**Inputs:** 
1. Bob's part of the entangled pair of qubits `qBob`.
2. The tuple of classical bits received from Alice, in the format used in Send Message exercise.
3. The PauliX, PauliY, or PauliZ basis in which the message qubit was originally prepared.

**Output:** 
A Bool indicating the eigenstate in which the message qubit was prepared, `One` as `true` and `Zero` as `false`. The state of the qubit `qBob` in the end of the operation doesn't matter.