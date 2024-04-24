To start, we will prepare the $\frac{1}{\sqrt{3}} \big( |00\rangle +  |01\rangle + |10\rangle \big)$ state using the solution to task 2.3. To get to the final state, we need to add the relative phases to both $|01\rangle$ and $|10\rangle$ basis states without changing the $|00\rangle$ state.

First, we want to transform the $|01\rangle$ state to the $\omega |01\rangle = e^{2\pi i/3} |01\rangle$ state, while not changing the other states.
Using the [$R_1$](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/r1) gate, we can change a qubit state from $|1\rangle$ to $e^{i\theta}|1\rangle$ without changing the $|0\rangle$ state.
Indeed, here is the effect of the $R_1$ gate on the $|0\rangle$ and the $|1\rangle$ states:

$$ R_1 |0\rangle = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} \cdot \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \begin{bmatrix} 1 \\ 0 \end{bmatrix} = |0\rangle $$

$$ R_1 |1\rangle = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} \cdot \begin{bmatrix} 0 \\ 1 \end{bmatrix} = \begin{bmatrix} 0 \\ e^{i\theta} \end{bmatrix} = e^{i\theta}|1\rangle $$

When we apply the $R_1$ gate to the second qubit, this will only affect the $|01\rangle$ term, which is exactly what we want. Knowing this, we just need the right value for $\theta$, which in this case is $\frac{2\pi}{3}$.

We use the same approach to change $|10\rangle$ term to $\omega^2 |10\rangle$. By applying the $R_1$ gate to the first qubit we will only change the $|10\rangle$ term. In this case the right $\theta$ will be $\frac{4\pi}{3}$.


@[solution]({
    "id": "superposition__three_states_two_qubits_phases_solution",
    "codePath": "./Solution.qs"
})
