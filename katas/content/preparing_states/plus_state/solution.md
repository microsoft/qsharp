When you start learning the basic quantum gates, one of the first gates described will be the Hadamard gate:

$$H = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix}$$

This gate converts $\ket{0}$ into $\ket{+} = \frac{1}{\sqrt{2}} \big(\ket{0} + \ket{1}\big)$ and $\ket{1}$ into $\ket{−} = \frac{1}{\sqrt{2}} \big(\ket{0} - \ket{1}\big)$.  The first of these transformations is exactly the one you're looking for!

Hadamard gate is one of the built-in gates in Q#, available in the `Microsoft.Quantum.Intrinsic` namespace.
It's open in any Q# source files by default, so you can use it right away.

@[solution]({
    "id": "preparing_states__plus_state_solution",
    "codePath": "./Solution.qs"
})
