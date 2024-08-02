The required transformation is actually just the expression for the transformation of an arbitrary state in the $Z$ basis (that is, the computational basis) to one in the $X$ basis (the Hadamard basis). 

For the starting state $\ket{0}$, $x_0=1$ and $x_1=0$, so the new state will be  $\frac1{\sqrt2} (\ket{0} + \ket{1}) = \ket{+}$.
Similarly, for the starting state $\ket{1}$, $x_0=0$ and $x_1=1$, so the new state will be  $\frac1{\sqrt2} (\ket{0} - \ket{1}) = \ket{-}$.

And you already know a gate that will do this: the Hadamard gate!

@[solution]({
"id": "qft__single_qubit_solution",
"codePath": "./Solution.qs"
})
