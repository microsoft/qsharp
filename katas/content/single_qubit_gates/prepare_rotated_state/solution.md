You use the rotation gate $R_x(\theta)$. This gate turns the state $\ket{0}$ into $R_x(\theta)\ket{0} = \cos\frac{\theta}{2}\ket{0} - i\sin\frac{\theta}{2}\ket{1}$.
This is similar to the state you need. You just need to find an angle $\theta$ such that $\cos\frac{\theta}{2}=\alpha$ and $\sin\frac{\theta}{2}=\beta$. You can use these two equations to solve for $\theta$: $\theta = 2\arctan\frac{\beta}{\alpha}$. (*Note: It's given that $\alpha^2 + \beta^2=1$*).
Hence the required gate is $R_x(2\arctan\frac{\beta}{\alpha})$, which in matrix form is $\begin{bmatrix} \alpha & -i\beta \\ -i\beta & \alpha \end{bmatrix}$.
This gate turns $\ket{0} = \begin{bmatrix} 1 \\ 0\end{bmatrix}$ into $\begin{bmatrix} \alpha & -i\beta \\ -i\beta & \alpha \end{bmatrix} \begin{bmatrix} 1 \\ 0\end{bmatrix} = \begin{bmatrix} \alpha \\ -i\beta \end{bmatrix} = \alpha\ket{0} -i\beta\ket{1}$.

> Trigonometric functions are available in Q# via the `Std.Math` namespace. In this case, you'll need <a href="https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.math/arctan2" target="_blank">ArcTan2</a>.

@[solution]({
    "id": "single_qubit_gates__prepare_rotated_state_solution",
    "codePath": "./Solution.qs"
})
