Since the $X$ gate is self-adjoint, you know that its eigenvalues can only be $+1$ and $-1$.
Now, you need to find the eigenvectors that correspond to these eigenvalues. 
To do this, you need to solve the two equations:
- $X \begin{bmatrix} v_0 \\ v_1 \end{bmatrix} = \begin{bmatrix} v_0 \\ v_1 \end{bmatrix}$, which gives you $v_0 = v_1$.
- $X \begin{bmatrix} v_0 \\ v_1 \end{bmatrix} = -\begin{bmatrix} v_0 \\ v_1 \end{bmatrix}$, which gives you $v_0 = -v_1$.

One of the eigenvectors should consist of two equal elements, and the other - of two elements with equal absolute values but opposite signs.

@[solution]({
    "id": "phase_estimation__eigenvectors_x_solution", 
    "codePath": "Solution.qs"
})
