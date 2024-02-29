**Input**: 
1. A 2-qubit system in the state
$$
|\psi\rangle = \frac{1}{\sqrt{2}} |0\rangle \otimes ( a |0\rangle + b|1\rangle) + \frac{1}{\sqrt{2}} |1\rangle \otimes (b|0\rangle + a |1\rangle),
$$
where the constants $a$ and $b$ satisfying $|a|^2 + |b|^2 = 1$ are unknown.
2. An integer $ind$ which is either $0$ or $1$.

**Goal**: 
- If $ind$ equals 0, convert the state of the second qubit into $a|0\rangle + b|1\rangle$
- If $ind$ equals 1, convert the state of the second qubit into $b|0\rangle + a|1\rangle$. 

The state of the first qubit at the end does not matter (it has to be not entangled with the second qubit).
