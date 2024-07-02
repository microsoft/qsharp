> As a reminder, $$H = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix}$$

The solution relies on the well-known identity $HXH = Z$ (i.e., Hadamard gate can be used to convert $X$ to $Z$ and vice versa). Applying a sequence of gates "given unitary - $X$ - given unitary" will be equivalent to $XXX = X$ gate if the unitary was $X$, and to $HXH = Z$ if the unitary was $H$. With this observation we need to distinguish $X$ from $Z$, which can be done using the $\ket{0}$ state: $X$ gate will change it to $\ket{1}$, and $Z$ gate will leave it unchanged.

@[solution]({
    "id": "distinguishing_unitaries__h_x_solution",
    "codePath": "Solution.qs"
})
