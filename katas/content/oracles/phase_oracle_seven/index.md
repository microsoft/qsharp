**Input:**
  3 qubits in an arbitrary state $|x\rangle$ (input/query register).

**Goal:**

Flip the sign of the input state $|x\rangle$ if the input register is in
the state $|111\rangle$ (encoding the integer $7$), and leave the input register unchanged otherwise.  
Don't allocate extra qubits to perform this operation.

**Examples:**

* If the query register is in the state $|111\rangle$, flip its sign.
* If the query register is in the state $|010\rangle$ or $|101\rangle$, do nothing.

<details>
  <summary><b>Need a hint?</b></summary>
  To solve this problem, you need to find a gate that will only flip the sign of the $|111\rangle$ basis state.  Which single-qubit gate flips the sign of the basis state $|1\rangle$ but not $|0\rangle$? How can you modify this gate to solve this problem?
</details>
