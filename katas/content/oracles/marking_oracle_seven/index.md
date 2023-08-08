**Inputs:**

  1. 3 qubits in an arbitrary state $|x\rangle$ (input/query register)
    
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit)

**Goal:**

Flip the state of $|y\rangle$ if the input register is in the 
state $|111\rangle$, and leave the state $|y\rangle$ unchanged otherwise.

**Examples:**

* If the query register is in the state $|111\rangle$, flip the state of the target qubit $|y\rangle$.
* If the query register is in the state $|010\rangle$ or $|101\rangle$, do nothing.
