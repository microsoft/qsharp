We use the rotation gate $R_x(\theta)$. This gate turns the state $|0\rangle$ into $R_x(\theta)|0\rangle = \cos\frac{\\theta}{2}|0\rangle - i\sin\frac{\theta}{2}|1\rangle$.
This is similar to the state we need. We just need to find an angle $\theta$ such that $\cos\frac{\theta}{2}=\alpha$ and $\sin\frac{\theta}{2}=\beta$. We can use these two equations to solve for $\theta$: $\theta = 2\arctan\frac{\beta}{\alpha}$. (*Note: It is given that $\alpha^2 + \beta^2=1$*).
Hence the required gate is $R_x(2\arctan\frac{\beta}{\alpha})$, which in matrix form is $\begin{bmatrix} \alpha & -i\beta \\\ -i\beta & \alpha \end{bmatrix}$.
This gate turns $|0\rangle = \begin{bmatrix} 1 \\\ 0\end{bmatrix}$ into $\begin{bmatrix} \alpha & -i\beta \\\ -i\beta & \alpha \end{bmatrix} \begin{bmatrix} 1 \\\ 0\end{bmatrix} = \begin{bmatrix} \alpha \\\ -i\beta \end{bmatrix} = \alpha|0\rangle -i\beta|1\rangle$.

> Trigonometric functions are available in Q# via the [Math](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math) namespace. In this case we will need [ArcTan2](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arctan2).

@[solution]({
    "id": "prepare_rotated_state_solution",
    "codePath": "./Solution.qs"
})
