We know that:

$$
R1(\alpha)=
 \begin{bmatrix}1 & 0 \\ 0 & \color{red}{e^{i\alpha}}\end{bmatrix}
$$

So we have:

$$
R1(\beta \ket{0} + \gamma \ket{1}) =
 \begin{bmatrix}1 & 0 \\ 0 & \color{red}{e^{i\alpha}} \end{bmatrix}
 \begin{bmatrix}\beta \\ \gamma \\ \end{bmatrix}=
\begin{bmatrix}1 \cdot \beta + 0 \cdot \gamma \\ 0 \cdot \beta + {\color{red}{e^{i\alpha}}} \cdot \gamma \end{bmatrix}=
 \begin{bmatrix}\beta \\ {\color{red}{e^{i\alpha}}} \gamma \end{bmatrix}=
 \beta \ket{0} + {\color{red}{e^{i\alpha}}} \gamma \ket{1}
$$

> Suppose now that $\alpha = \frac{\pi}{2}$.
> Then $e^{i\alpha}= \cos\frac{\pi}{2} + i\sin\frac{\pi}{2}$.
> And, since $\cos\frac{\pi}{2}= 0$ and $\sin\frac{\pi}{2} = 1$, then we have that $\cos\frac{\pi}{2} + i \sin\frac{\pi}{2} = i$, and  
> $R1(\frac{\pi}{2}) = S$, which we used in the second solution to the task "Relative Phase i".

@[solution]({
    "id": "single_qubit_gates__complex_phase_solution",
    "codePath": "./Solution.qs"

})
