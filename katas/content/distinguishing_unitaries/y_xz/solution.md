> As a reminder, $$Y = \begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix} = iXZ$$

We see that these two gates differ by a global phase $i = e^{i\pi}$. Applying the gates twice will give us gates $Y^2 = I$ and $(XZ)^2 = XZXZ = -XXZZ = -I$, respectively.

Now we need to distinguish $I$ gate from $-I$ gate, which is the same thing we did in the previous task.

@[solution]({
    "id": "distinguishing_unitaries__y_xz_solution",
    "codePath": "Solution.qs"
})
