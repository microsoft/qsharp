**Inputs:**

1. `qAlice` : Qubit received from Alice.
2. `qBob` : Bob's part of the entangled pair.

**Goal** :  Decode the message using the qubit received from Alice and reset both qubits to a $\ket{00}$ state. For this, retrieve two bits of classic data from the qubits and return a tuple of the form `(Bool, Bool)` to represent `message`. The state of the qubits in the end of the operation should be $\ket{00}$.
