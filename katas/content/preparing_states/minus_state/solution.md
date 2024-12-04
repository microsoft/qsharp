As you've seen in the previous task, the Hadamard gate maps the basis state $\ket{0}$ to $\frac{1}{\sqrt2}\big(\ket{0} + \ket{1}\big)$ and $\ket{1}$ to $\frac{1}{\sqrt2}\big(\ket{0} - \ket{1}\big)$. 
If your qubit was already in the $\ket{1}$ state, you would simply apply the Hadamard gate to prepare the required $\ket{-}$ state. 
Fortunately, there is another operation you can use to change the state $\ket{0}$ to $\ket{1}$, namely the $X$ gate:

$$X = \begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$$

This gate transforms $\ket{0} \longmapsto \ket{1}$ and $\ket{1} \longmapsto \ket{0}$.
$X$ is another one of the built-in gates in Q# from the `Std.Intrinsic` namespace.

Thus, your solution should apply the $X$ gate to your qubit, followed by the Hadamard gate.

@[solution]({
    "id": "preparing_states__minus_state_solution",
    "codePath": "./Solution.qs"
})
