**Input**: 
1. A 2-qubit system in the state
$$
\ket{\psi} = \frac{1}{\sqrt{2}} \ket{0} \otimes ( a \ket{0} + b\ket{1}) + \frac{1}{\sqrt{2}} \ket{1} \otimes (b\ket{0} + a \ket{1}),
$$
where the constants $a$ and $b$ satisfying $|a|^2 + |b|^2 = 1$ are unknown.
2. An integer $ind$ which is either $0$ or $1$.

**Goal**: 
- If $ind$ equals 0, convert the state of the second qubit into $a\ket{0} + b\ket{1}$
- If $ind$ equals 1, convert the state of the second qubit into $b\ket{0} + a\ket{1}$. 

The state of the first qubit at the end does not matter (it has to be not entangled with the second qubit).
