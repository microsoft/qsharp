When you start learning the basic quantum gates, one of the first gates described will be the Hadamard gate:

$$H = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix}$$

This gate converts $|0\rangle$ into $|+\rangle = \frac{1}{\sqrt{2}} \big(|0\rangle + |1\rangle\big)$ and $|1\rangle$ into $|−\rangle = \frac{1}{\sqrt{2}} \big(|0\rangle - |1\rangle\big)$.  The first of these transformations is exactly the one we're looking for!

Hadamard gate is one of the built-in gates in Q#, available in the `Microsoft.Quantum.Intrinsic` namespace.
It is open in any Q# source files by default, so you can use it right away.

@[solution]({
    "id": "superposition__plus_state_solution",
    "codePath": "./Solution.qs"
})
