> As a reminder, $$Z = \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$$

We won't be able to distinguish $I$ from $Z$ by applying them to the basis states, since they both leave the $\ket{0}$ state unchanged and add a phase to the $\ket{1}$ state: 

$$I\ket{0} = \ket{0}, I\ket{1} = \ket{1}$$
$$Z\ket{0} = \ket{0}, Z\ket{1} = -\ket{1}$$

However, if we try applying these gates to a superposition of basis states, we'll start seeing a difference between the resulting states:

$$I \big(\frac{1}{\sqrt2}(\ket{0} + \ket{1})\big) = \frac{1}{\sqrt2}(\ket{0} + \ket{1})$$
$$Z \big(\frac{1}{\sqrt2}(\ket{0} + \ket{1})\big) = \frac{1}{\sqrt2}(\ket{0} - \ket{1})$$

These two states are orthogonal and can be distinguished by measuring them in the $\{ \ket{+}, \ket{-}\}$ basis using [`MResetX`](https://learn.microsoft.com/en-us/qsharp/api/qsharp-lang/microsoft.quantum.measurement/mresetx) operation (which is equivalent to applying an $H$ gate and measuring in the computational basis).

> The task of distinguishing these two states is covered in more detail in the Distinguishing Quantum States kata.

@[solution]({
    "id": "distinguishing_unitaries__i_z_solution",
    "codePath": "Solution.qs"
})
