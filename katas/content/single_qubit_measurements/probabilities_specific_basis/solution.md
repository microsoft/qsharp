1. To find the probabilities of measuring $+$ and $-$, we first need to express the state $\ket 0$ in terms of $\ket +$ and $\ket -$. Using the fact that $\ket{\pm} = \frac{1}{\sqrt{2}}  (\ket{0} \pm \ket{1})$, we can show that 
$$
\ket 0 = \frac{1}{\sqrt{2}} \ket{+} + \frac{1}{\sqrt{2}} \ket{-}.
$$
Thus, the probability of measuring $+$ is $|\frac1{\sqrt2}|^2 = 0.5$, and similarly, the probability of measuring $-$ is $0.5$.

2. Similar to the first part, we need to express the state $\ket \psi = 0.6 \ket 0 + 0.8 \ket 1$ in the $\ket{\pm i}$ basis. For this calculation, we use the projection matrix approach.
First, we recall that the states $\ket{\pm i}$ are given by
$$
\ket{\pm i} = \frac1{\sqrt2} (\ket 0 \pm i \ket 1).
$$
We can now construct the two projectors $P_{\pm i}$ onto states $\ket {\pm i}$ as follows:
\begin{align}
P_{i} &= \ket{i}\bra{i} = \frac{1}{2} \begin{bmatrix} 1 \\ i \end{bmatrix} \begin{bmatrix} 1 & -i \end{bmatrix} = \frac{1}{2} \begin{bmatrix}1 & -i \\ i & 1\end{bmatrix}; \\
P_{-i} &=\ket{-i}\bra{-i} = \frac{1}{2} \begin{bmatrix} 1 \\ -i \end{bmatrix} \begin{bmatrix} 1 & i \end{bmatrix} = \frac{1}{2} \begin{bmatrix}1 & i \\ -i & 1\end{bmatrix}
\end{align}
Recalling that the probabilities of measuring $\pm i$ are equal to the norm of the vectors $P_{\pm i}\ket \psi$, we now apply $P_{\pm i}$ to $\ket \psi$:
\begin{align}
P_{+i} \ket \psi &= \frac{1}{2} \begin{bmatrix}1 & -i \\ i & 1\end{bmatrix} \begin{bmatrix} 0.6 \\ 0.8 \end{bmatrix} = \frac{1}{2} \begin{bmatrix} 0.6 - 0.8i \\ 0.8 + 0.6i \end{bmatrix} ;\\
P_{-i} \ket \psi &= \frac{1}{2} \begin{bmatrix}1 & i \\ -i & 1\end{bmatrix} \begin{bmatrix} 0.6 \\ 0.8 \end{bmatrix} = \frac{1}{2} \begin{bmatrix} 0.6 + 0.8i \\ 0.8 - 0.6i \end{bmatrix}.
\end{align}
Hence, the probabilities of measuring $\pm i$, which we denote by $p(\pm i)$, are
\begin{align}
p(+i)& = |P_{+i} \ket \psi|^2 = \frac{1}{4}(|0.6 - 0.8i|^2 + |0.8 + 0.6i|^2) = \frac{1}{2}; \\
p(-i)& = |P_{-i} \ket \psi|^2 = \frac{1}{4}(|0.6 + 0.8i|^2 + |0.8 - 0.6i|^2) = \frac{1}{2}.
\end{align}
