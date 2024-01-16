**Input:** A two-qubit system in the basis state $|00\rangle = \begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix}$.

**Goal:** Transform the system into the state $\frac{1}{\sqrt2}\big(|00\rangle - |10\rangle\big) = \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ 0 \\\ -1 \\\ 0 \end{bmatrix}$.

> Note that the quantum states are represented in little-endian format For a state $|qs_1 qs_0\rangle$, the "first" qubit (`qs[0]` in a Q# qubit register) refers to the least-significant qubit $qs_0$, while the "second" qubit (`qs[1]` in a Q# register) refers to the most-significant qubit $qs_1$.

<details>
    <summary><b>Need a hint?</b></summary>
    Represent the target state as a tensor product $\frac{1}{\sqrt2}\big(|0\rangle - |1\rangle\big) \otimes |0\rangle = \frac{1}{\sqrt2}\begin{bmatrix} 1 \\\ -1 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix}$.
</details>
