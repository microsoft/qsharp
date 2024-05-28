> As a reminder, $$S = \begin{bmatrix} 1 & 0 \\ 0 & i \end{bmatrix}$$

This task differs from the previous two in that it allows you to apply the given unitary **twice**. 
Let's treat this as a hint that it is, and check how the given gates looks when applied twice. 
If you square the corresponding matrices (which is quite simple to do for diagonal matrices), you'll get

$$Z^2 = \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix} = I, S^2 = \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix} = Z$$

This means that the task of identifying the *square* of the given unitary transformation is the same as distinguishing $I$ from $Z$ gates - and that's exactly the previous task.

@[solution]({
    "id": "distinguishing_unitaries__z_s_solution",
    "codePath": "Solution.qs"
})
