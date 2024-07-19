The state of single qubit can be represented as a two-dimensional column vector $\begin{bmatrix} \alpha \\ \beta \end{bmatrix}$, where $\alpha$ and $\beta$ are complex numbers that satisfy $|\alpha|^2 + |\beta|^2 = 1$. When you measure the qubit, you get either 0 with probability $|\alpha|^2$ or 1 with probability $|\beta|^2$. Essentially we can control probability of measurement outcome by setting the right amplitudes of basis states. 

When you allocate the qubit in Q#, amplitudes $\alpha$ and $\beta$ are 1 and 0, respectively. Now your goal is set equal amplitudes for $\alpha$ and $\beta$ for absolute randomness. You can achieve that by simply applying Hadamard gate to the initial state $\ket{0}$:

$$
H\ket{0} =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix}
\begin{bmatrix} 1 \\ 0 \end{bmatrix} =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 \cdot 1 + 1 \cdot 0 \\ 1 \cdot 1 + (-1) \cdot 0 \end{bmatrix} =
\frac{1}{\sqrt{2}}
\begin{bmatrix} 1 \\ 1 \end{bmatrix}
$$

Now, both 0 and 1 measurement outcomes occur with equal probability of $|\frac{1}{\sqrt{2}}|^2 = \frac{1}{2}$.

> Since the outcome probability is the square of the absolute value of amplitude, you'll get the same distribution of outcomes by measuring the $\ket{-}$ state, which you can prepare by applying the Hadamard gate to the basis state $\ket{1}$. Try it out to achieve the stretch goal!

@[solution]({
    "id": "random_numbers__random_bit_solution",
    "codePath": "Solution.qs"
})
