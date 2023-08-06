The state of single qubit can be represented as a two-dimensional column vector $\begin{bmatrix} \alpha\\ \beta \end{bmatrix}$, where $\alpha$ and $\beta$ are complex numbers that satisfy $|\alpha|^2 + |\beta|^2 = 1$. When we measure the qubit, we get either 0 with probability $|\alpha|^2$ or 1 with probability $|\beta|^2$. Essentially we can control probablity of measurement outcome by setting the right amplitudes of basis states. 

When we allocate the qubit in Q#, amplitudes $\alpha$ and $\beta$ are 1 and 0, respectively. Now our goal is set equal amplitudes for $\alpha$ and $\beta$ for absolute randomness. We can achieve that by simply applying Hadamard gate to the initial state $|0\rangle$:

$$
H|0\rangle =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix}
\begin{bmatrix} 1 \\\ 0 \end{bmatrix} =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 \cdot 1 + 1 \cdot 0 \\\ 1 \cdot 1 + (-1) \cdot 0 \end{bmatrix} =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 \\\ 1 \end{bmatrix}
$$

Now, both 0 and 1 measurement outcomes occur with equal probablity of $|\frac{1}{\sqrt{2}}|^2 = \frac{1}{2}$.

> Note: Since probablity is the square of the absolute value of amplitude, we will get the same randomness by applying Hadamard gate on base state $|1\rangle$. Try it out as an exercise!

@[solution]({
    "id": "random_bit_solution",
    "codePath": "solution.qs"
})
