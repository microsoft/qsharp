What happens if you apply the unitary $U$ to the state $\ket{\psi}$? You end up in one of the states $\ket{\psi}$ or $-\ket{\psi}$, depending on whether the eigenvalue is $+1$ or $-1$. You need to distinguish these two scenarios, but they differ by a global phase, so you can't do this using just one qubit.

However, if you use the fact that you have access to controlled variant of $U$ and can allocate more than one qubit, you can use a variant of the phase kickback trick to distinguish these scenarios.

If you apply a controlled $U$ gate to two qubits: the control qubit in the $\ket{+}$ state and the target qubit in the state $\ket{\psi}$, you'll get the following state:

$$CU \ket{+}\ket{\psi} = \frac1{\sqrt2}(CU \ket{0}\ket{\psi} + CU \ket{1}\ket{\psi}) = \frac1{\sqrt2}(\ket{0}\ket{\psi} + \ket{1}U\ket{\psi}) = $$
$$= \frac1{\sqrt2}(\ket{0}\ket{\psi} + \ket{1}\lambda\ket{\psi}) = 
\begin{cases}
\ket{+}\ket{\psi} \textrm{ if } \lambda = 1 \\ 
\ket{-}\ket{\psi} \textrm{ if } \lambda = -1
\end{cases}$$

You only need to measure the control qubit in the Hadamard basis to figure out whether its state is $\ket{+}$ or $\ket{-}$, and you'll be able to tell the value of the eigenvalue $\lambda$.

@[solution]({
    "id": "phase_estimation__one_bit_eigenphase_solution", 
    "codePath": "Solution.qs"
})
