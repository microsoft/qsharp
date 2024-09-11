You'll start by putting the first qubit in the state $\alpha\ket{0} + \beta\ket{1}$, where $\alpha$ and $\beta$ are the square roots of relative weights of all basis states which start with 0 and with 1, respectively.  

In this case, the state can be represented as $\frac{1}{\sqrt{2}} \big( \ket{0} \otimes \ket{0} + \ket{1} \otimes \frac{1}{\sqrt2}(\ket{0}+\ket{1}) \big)$.

You see that the relative weights of $\ket{0}$ and $\ket{1}$ states of the first qubit are both $\frac12$ (the squares of their amplitudes in the decomposition above). This means that you can do the first step by applying the $H$ gate to the first qubit, which gives the $\frac{1}{\sqrt{2}}\ket{00} + \frac{1}{\sqrt{2}}\ket{10}$ state. In matrix form this will look as follows:  

$$ H \otimes I = \frac{1}{\sqrt{2}} \begin{bmatrix}1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 1 & 0 & -1 & 0 \\ 0 & 1 & 0 & -1 \end{bmatrix} \cdot
\begin{bmatrix} 1 \\ 0 \\0 \\ 0 \end{bmatrix} = \frac{1}{\sqrt{2}} \begin{bmatrix} 1 \\ 0 \\ 1 \\0 \end{bmatrix} = \frac{1}{\sqrt{2}} \big( \ket{0} + \ket{1} \big) \otimes \ket{0}$$

Now the first term of the state $\frac{1}{\sqrt2}\ket{00}$ matches that of the goal state, and you need to convert the second term $\frac{1}{\sqrt2}\ket{10}$ to $\ket{1} \otimes \frac{1}{\sqrt2}(\ket{0}+\ket{1})$.

To do this, you use the controlled $H$ gate. The matrix representation of the controlled $H$ gate is similar to the $CNOT$ gate, however the bottom right block of the matrix is not an $X$ gate but the $H$ gate:

$$\text{Controlled } H = \begin{bmatrix}1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & \frac{1}{\sqrt{2}} & \frac{1}{\sqrt{2}} \\ 0 & 0 & \frac{1}{\sqrt{2}} & -\frac{1}{\sqrt{2}} \end{bmatrix}$$

When this is applied to the current state, you get your goal state:

$$\text{Controlled } H \cdot \frac{1}{\sqrt{2}} \begin{bmatrix} 1 \\ 0 \\ 1 \\0 \end{bmatrix} = \begin{bmatrix}\frac{1}{\sqrt{2}} \\ 0 \\ \frac{1}{2} \\ \frac{1}{2} \end{bmatrix} = \frac{1}{\sqrt{2}}\ket{00}+\frac{1}{2}\ket{10}+\frac{1}{2}\ket{11}  $$

@[solution]({
    "id": "preparing_states__controlled_rotation_solution",
    "codePath": "./Solution.qs"
})
