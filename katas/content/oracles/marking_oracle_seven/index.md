**Inputs:**

  1. 3 qubits in an arbitrary state $\ket{x}$ (input/query register)
  2. A qubit in an arbitrary state $\ket{y}$ (target qubit)

**Goal:**

Flip the state of $\ket{y}$ if the input register is in the 
state $\ket{111}$, and leave the state $\ket{y}$ unchanged otherwise.

**Examples:**

* If the query register is in the state $\ket{111}$, flip the state of the target qubit $\ket{y}$.
* If the query register is in the state $\ket{010}$ or $\ket{101}$, do nothing.
