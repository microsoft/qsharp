**Input:** $N \ge 2$ qubits (stored in an array of length $N$) which are guaranteed to be either in the GHZ state - an even superposition of $\ket{0...0}$ and $\ket{1...1}$ states - or in the W state we saw in the previous task. (For example, for $N = 3$ GHZ state is $\frac1{\sqrt2}(\ket{000} + \ket{111})$.

**Output:**
* 0 if the qubits were in the GHZ state,
* 1 if they were in the W state.

The state of the qubits at the end of the operation does not matter.
