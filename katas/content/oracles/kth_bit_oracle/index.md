**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. An integer $k$ such that $0 \leq k < N$.

**Goal:**

Flip the sign of the input state $|x\rangle$ if the $k$-th bit of $x$ is $1$.  
**Implement this oracle without using auxiliary qubits.**

**Examples:**

* If the query register is in the state $|010\rangle$ and $k=0$, do nothing.
* If the query register is in the state $|010\rangle$ and $k=1$, flip the sign of the basis state.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a phase oracle, because we are changing the phase of the input state $|x\rangle$ based on the value of the function $f(x)$.
</details>
