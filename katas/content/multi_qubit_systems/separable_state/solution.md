### Solution

To separate the state into a tensor product of two single-qubit states, we need to represent it in the following way:

$$\begin{bmatrix} \alpha \color{red}\gamma \\ \alpha \color{red}\delta \\ \beta \color{red}\gamma \\ \beta \color{red}\delta \end{bmatrix} = \begin{bmatrix} \alpha \\ \beta \end{bmatrix} \otimes \begin{bmatrix} \color{red}\gamma \\ \color{red}\delta \end{bmatrix}$$

This brings us to a system of equations:

$$\begin{cases}
\alpha\gamma = \frac{1}{2} \\
\alpha\delta = \frac{i}{2} \\
\beta \gamma = \frac{-i}{2} \\
\beta \delta = \frac{1}{2} \\
\end{cases}$$

Solving this system of equations gives us the answer:

$$\alpha = \frac{1}{\sqrt2}, \beta = \frac{-i}{\sqrt2}, \gamma = \frac{1}{\sqrt2}, \delta = \frac{i}{\sqrt2}$$

$$\frac{1}{2} \begin{bmatrix} 1 \\ i \\ -i \\ 1 \end{bmatrix} = \frac{1}{\sqrt2}
\begin{bmatrix} 1 \\ -i \end{bmatrix} \otimes \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ i \end{bmatrix}$$


> Note that finding such representation is not always possible, as you will see in the next exercise.
