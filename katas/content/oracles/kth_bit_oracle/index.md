**Inputs:**

  1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
  2. An integer $k$ such that $0 \leq k < N$.

**Goal:**

Flip the sign of the input state $\ket{x}$ if the $k$-th bit of $x$ is $1$.  
*Implement this oracle without using auxiliary qubits.*

**Examples:**

* If the query register is in the state $\ket{010}$ and $k=0$, do nothing.
* If the query register is in the state $\ket{010}$ and $k=1$, flip the sign of the basis state.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?</b></summary>
    This is a phase oracle, because you're changing the phase of the input state $\ket{x}$ based on the value of the function $f(x)$.
</details>
