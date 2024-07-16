To start, we will prepare the $\frac{1}{\sqrt{3}} \big( \ket{00} +  \ket{01} + \ket{10} \big)$ state using the solution to the previous task. To get to the final state, we need to add the relative phases to both $\ket{01}$ and $\ket{10}$ basis states without changing the $\ket{00}$ state.

First, we want to transform the $\ket{01}$ state to the $\omega \ket{01} = e^{2\pi i/3} \ket{01}$ state, while not changing the other states.
Using the [$R_1$](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/r1) gate, we can change a qubit state from $\ket{1}$ to $e^{i\theta}\ket{1}$ without changing the $\ket{0}$ state.
Indeed, here is the effect of the $R_1$ gate on the $\ket{0}$ and the $\ket{1}$ states:

$$ R_1 \ket{0} = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} \cdot \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \ket{0} $$

$$ R_1 \ket{1} = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} \cdot \begin{bmatrix} 0 \\ 1 \end{bmatrix} = \begin{bmatrix} 0 \\ e^{i\theta} \end{bmatrix} = e^{i\theta}\ket{1} $$

When we apply the $R_1$ gate to the second qubit, this will only affect the $\ket{01}$ term, which is exactly what we want. Knowing this, we just need the right value for $\theta$, which in this case is $\frac{2\pi}{3}$.

We use the same approach to change $\ket{10}$ term to $\omega^2 \ket{10}$. By applying the $R_1$ gate to the first qubit we will only change the $\ket{10}$ term. In this case the right $\theta$ will be $\frac{4\pi}{3}$.


@[solution]({
    "id": "preparing_states__three_states_two_qubits_phases_solution",
    "codePath": "./Solution.qs"
})
