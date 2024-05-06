**Input:** $N$ qubits in the $|0 \dots 0\rangle$ state ($N$ is not necessarily a power of two).

**Goal:**  Change the state of the qubits to the [W state](https://en.wikipedia.org/wiki/W_state) - an equal superposition of $N$ basis states on $N$ qubits which have Hamming weight of 1.

For example, for $N = 3$ the required state is $\frac{1}{\sqrt{3}}\big(|100\rangle + |010\rangle + |001\rangle\big)$.

<details>
  <summary><b>Need a hint?</b></summary>
  You can modify the signature of the given operation to specify its controlled specialization.
</details>
