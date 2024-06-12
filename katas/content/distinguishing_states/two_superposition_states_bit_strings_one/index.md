**Inputs:**

1. $N$ qubits (stored in an array of length $N$) which are guaranteed to be in one of the two superposition states described by the given arrays of bit strings.
2. Two arrays of bit strings represented as `Bool[][]`s.
The arrays have dimensions $M_1 \times N$ and $M_2 \times N$
 respectively, where $N$ is the number of qubits and $M_1$ and $M_2$ are the numbers of bit strings in each array. Note that in general $M_1 \neq M_2$.
An array of bit strings `[b₁, ..., bₘ]` defines a state that is an equal superposition of all basis states defined by bit strings $b_1, ..., b_m$.
For example, an array of bit strings `[[false, true, false], [false, true, true]]` defines a superposition state $\frac{1}{\sqrt2}\big(\ket{010} + \ket{011}\big)$.

You are guaranteed that there exists an index of a qubit Q for which:

* all the bit strings in the first array have the same value in this position (all `bits1[j][Q]` are the same),
* all the bit strings in the second array have the same value in this position (all `bits2[j][Q]` are the same),
* these values are different for the first and the second arrays.

**Output:**

* 0 if qubits were in the superposition state described by the first array,
* 1 if they were in the superposition state described by the second array.

> For example, for arrays `[[false, true, false], [false, true, true]]` and `[[true, false, true], [false, false, true]]` return 0 corresponds to state $\frac{1}{\sqrt2}\big(\ket{010} + \ket{011}\big)$, return 1 corresponds to state $\frac{1}{\sqrt2}\big(\ket{101} + \ket{001}\big)$, and you can distinguish these states perfectly by measuring the second qubit.

**You are allowed to use exactly one measurement.** The state of the qubits at the end of the operation does not matter.
