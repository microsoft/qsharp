Transform Bob's qubit into the required state using the two classical bits received from Alice and measure it in the same basis in which she prepared the message.

**Inputs:** 
1. Bob's part of the entangled pair of qubits `qBob`.
2. The tuple of classical bits received from Alice, in the format used in the Send Message exercise.
3. The basis in which the message qubit was originally prepared - `PauliX`, `PauliY`, or `PauliZ`.

**Output:** 
A Result value indicating the basis state in which the message qubit was prepared, `One` or `Zero`. The state of the qubit `qBob` in the end of the operation doesn't matter.