## Solution

The first step towards identifying the outcomes and their probabilities for joint measurements is to identify the eigenvectors corresponding to eigenvalues $\pm1$ of the Pauli operator. We note that since $X\ket{\pm}= \pm\ket{\pm}$, we have 
\begin{align}
XX \ket{++} &= \ket{++}, &XX \ket{--} &= \ket{--};\\
XX \ket{+-} &= -\ket{+-}, &XX \ket{-+} &= -\ket{-+}.
\end{align}
Thus, the $XX$ operator measures the parity in the Hadamard, or the $\ket{\pm}$ basis. That is, it distinguishes basis states with an even number of $+$'s from basis states which have an odd number of $+$'s.

The projector corresponding to a result of `Zero` is given by $P_{+1} = \ket{++}\bra{++} + \ket{--}\bra{--}$, while the projector corresponding to a result of `One` is given by $P_{-1} = \ket{+-}\bra{+-} + \ket{-+}\bra{-+}$. Then, we note that $P_{+1}$ annihilates states with odd parity, while leaving states with even parity unaffected. That is, for any values of the constants 
\begin{align}
P_{+1} ( \gamma \ket{++} + \delta \ket{--} ) &= ( \gamma \ket{++} + \delta \ket{--} )\\
P_{+1} ( \mu \ket{-+} + \nu \ket{+-} ) &= 0.
\end{align}
Similarly, $P_{-1}$ annihilates states with even parity, while leaving states with odd parity unaffected.


Now we express the given state in the Hadamard basis. We note that it is possible to go from the computational basis to the Hadamard basis using the following relations
$$
\ket{0} = \frac{1}{\sqrt{2}} \left( \ket{+} + \ket{-} \right)\\
\ket{1} = \frac{1}{\sqrt{2}} \left( \ket{+} - \ket{-} \right).
$$
Using these, we obtain
$$ \alpha |00\rangle + \beta |01\rangle + \beta |10\rangle + \alpha |11\rangle = (\alpha + \beta) |++\rangle + (\alpha - \beta) |--\rangle.$$
Thus, this state has an even parity in the Hadamard basis. It follows that an $XX$ Pauli measurement will result in the outcome `Zero` with probability 1, leaving the state unchanged after the measurement.


