The three-fourths phase gate above can be expressed as a product of two canonical gates - the $T$ gate is $\begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/4} \end{bmatrix}$ and the $S$ gate is $\begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/2} \end{bmatrix}$.

$$
\begin{bmatrix} 1 & 0 \\\ 0 & e^{i3\pi/4} \end{bmatrix} = 
\begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/4} \end{bmatrix} \begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/2} \end{bmatrix} = 
\begin{bmatrix} 1 & 0 \\\ 0 & e^{i\pi/4} \end{bmatrix} \begin{bmatrix} 1 & 0 \\\ 0 & i \end{bmatrix} = 
TS
$$

Note that $TS = ST$, so it doesn't matter in what order those gates are applied.

@[solution]({
    "id": "single_qubit_gates__three_quarters_pi_solution",
    "codePath": "./Solution.qs"
})
