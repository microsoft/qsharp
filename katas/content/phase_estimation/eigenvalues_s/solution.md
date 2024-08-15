Since the $S$ gate is diagonal, it's easy to realize that its eigenvectors are the basis vectors $\ket{0}$ and $\ket{1}$.

To find the corresponding eigenvalues, you need to solve the two equations:
- $S\ket{0} = \lambda \ket{0}$, which gives you $\lambda = 1$.
- $S\ket{1} = \lambda \ket{1}$, which gives you $\lambda = i$.

@[solution]({
    "id": "phase_estimation__eigenvalues_s_solution", 
    "codePath": "Solution.qs"
})
