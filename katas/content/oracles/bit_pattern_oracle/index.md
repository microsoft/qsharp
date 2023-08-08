**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit).
  3. A boolean array of length $N$ `pattern` representing a basis state; `true` and `false` elements correspond to $|1\rangle$ and $|0\rangle$, respectively.

**Goal:**

Flip the state of $|y\rangle$ if the input register matches the basis state
represented by `pattern`.  

**Examples:**

* If the query register is in the state $|010\rangle$ and `pattern = [false, true, false]`, flip the state $|y\rangle$.
* If the query register is in the state $|1001\rangle$ and `pattern = [false, true, true, false]`, do nothing.
    
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the input $|x\rangle$.
</details>

<br/>
<details>
  <summary><b>Need a hint?</b></summary>
  You need to flip the state of $|y\rangle$ if $|x\rangle$ matches the given pattern.  You may find the Q# library function <a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonbitstring">ControlledOnBitString</a> useful in your implementation.
</details>
