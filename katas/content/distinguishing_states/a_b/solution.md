We take a similar approach to the previous task: figure out a way to prepare the input states from the basis states and apply adjoint of that preparation before measuring the qubit.

To create the input states $\ket{A}$ and $\ket{B}$, a $R_y$ gate with $\theta = 2\alpha$ was applied to the basis states $\ket{0}$ and $\ket{1}$. As a reminder, 

$$R_y(\theta) = \begin{bmatrix} \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\ \sin\frac{\theta}{2} & \cos\frac{\theta}{2} \end{bmatrix}$$

We can return the input state to the basis state by applying $R_y$ gate with $-2\alpha$ as the rotation angle parameter to the input qubit.

The measurement in Pauli $Z$ basis gives two possibilities: 
1. The qubit is measured as $\ket{1}$, the input state was $\ket{B}$, we return `false`.
2. The qubit is measured as $\ket{0}$, the input state was $\ket{A}$, we return `true`.

@[solution]({
    "id": "distinguishing_states__a_b_solution",
    "codePath": "Solution.qs"
})
