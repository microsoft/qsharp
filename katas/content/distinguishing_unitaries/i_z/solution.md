> As a reminder, $$Z = \begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$$

We won't be able to distinguish $I$ from $Z$ by applying them to the basis states, since they both leave the $|0\rangle$ state unchanged and add a phase to the $|1\rangle$ state: 

$$I|0\rangle = |0\rangle, I|1\rangle = |1\rangle$$
$$Z|0\rangle = |0\rangle, Z|1\rangle = -|1\rangle$$

However, if we try applying these gates to a superposition of basis states, we'll start seeing a difference between the resulting states:

$$I \big(\frac{1}{\sqrt2}(|0\rangle + |1\rangle)\big) = \frac{1}{\sqrt2}(|0\rangle + |1\rangle)$$
$$Z \big(\frac{1}{\sqrt2}(|0\rangle + |1\rangle)\big) = \frac{1}{\sqrt2}(|0\rangle - |1\rangle)$$

These two states are orthogonal and can be distinguished by measuring them in the $\{ |+\rangle, |-\rangle\}$ basis using [`MResetX`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.measurement/mresetx) operation (which is equivalent to applying an $H$ gate and measuring in the computational basis).

> The task of distinguishing these two states is covered in more detail in the Distinguishing Quantum States kata.

@[solution]({
    "id": "distinguishing_unitaries__i_z_solution",
    "codePath": "Solution.qs"
})
