**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. An integer $k$ such that $0 \leq k < N$.

**Goal:**

Flip the sign of the basis state $|x\rangle$ if any of the bits of $x$ (not considering the $k$-th bit) are $1$ in input register. In other words, the input register with the $k$-th qubit excluded should not be in the all zero state to flip the sign of the input register. The state of the $k$-th qubit does not affect the result.

Feel free to explore implementing this operation with or without auxiliary qubits.

**Examples:**

* If the query register is in the state $|010\rangle$ and $k=0$, flip the sign of the register.
* If the query register is in the state $|010\rangle$ and $k=1$, do nothing.

<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?</b></summary>
    This is a phase oracle, because we are changing the phase of the input state $|x\rangle$ based on the value of the function $f(x)$.
</details>

<details>
  <summary><b>Need a hint?</b></summary>
  You can use the previously implemented oracles if needed by copying the code.
  <br/>
  You can use <a href="https://docs.microsoft.com/en-us/azure/quantum/user-guide/language/expressions/itemaccessexpressions">array slicing</a> to get parts of the array before and after the $k$-th element.
</details>
