**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit).

**Goal:**

Flip the state of $|y\rangle$ if the input register is in any basis state
except for $|00...0\rangle$ (the all zero state).

**Examples:**

* If the query register is in the state $|10000001\rangle$, $|11101101\rangle$ or $|0010101\rangle$, flip the state $|y\rangle$.
* If the query register is in the state $|000\rangle$, do nothing.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the input $|x\rangle$.
</details>

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  You need to flip the state of $|y\rangle$ for every input except $|00...0\rangle$, or, alternatively, flip it unconditionally and then flip it for the $|00...0\rangle$ state.   You may find the Q# library function <a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonint">ControlledOnInt</a> useful in your implementation.
</details>
