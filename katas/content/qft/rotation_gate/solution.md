To avoid adding the extra global phase, we have to use a gate that does not modify the $\ket{0}$ state, and only impacts $\ket{1}$: that is, $U\ket{\psi} = \alpha\ket{0} + \beta \cdot U\ket{1}$.

The built-in [R1Frac gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/r1frac) does exactly that:

$$\textrm{R1Frac}(n,k) = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\pi n/2^{k}} \end{bmatrix} $$

We specify $n=2$ to get the transformation required: 

$$\textrm{R1Frac}(2,k) = \begin{bmatrix} 1 & 0 \\ 0 & e^{2\pi i/2^{k}} \end{bmatrix} $$

@[solution]({
"id": "qft__rotation_gate_solution",
"codePath": "./Solution.qs"
})
