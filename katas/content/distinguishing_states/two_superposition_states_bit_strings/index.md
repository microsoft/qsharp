**Inputs:**

* $N$ qubits (stored in an array of length $N$) which are guaranteed to be in one of the two superposition states described by the given arrays of bit strings.
* Two arrays of bit strings represented as `Bool[][]`s.
  The arrays describe the superposition states in the same way as in the previous task, i.e. they have dimensions $M_1 \times N$ and $M_2 \times N$ respectively, $N$ being the number of qubits.

The only constraint on the bit strings is that **all bit strings in the two arrays are distinct**.

**Output:**
* 0 if qubits were in the superposition state described by the first array,
* 1 if they were in the superposition state described by the second array.

> For example, for bit strings `[[false, true, false], [false, false, true]]` and `[[true, true, true], [false, true, true]]` return 0 corresponds to state 
$\frac{1}{\sqrt2}\big(\ket{010} + \ket{001}\big)$, 
return 1 to state 
$\frac{1}{\sqrt2}\big(\ket{111} + \ket{011}\big)$.

**You can use as many measurements as you wish**. The state of the qubits at the end of the operation does not matter.
