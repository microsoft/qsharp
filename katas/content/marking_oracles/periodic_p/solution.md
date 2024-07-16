This problem is similar to the previous one: in that problem you had to compare the states of the qubits in symmetrical positions, and in this one you have to compare the states of the qubits at a distance $P$ from each other.
For the basis state $\ket{x_0 ... x_{N-1}}$ to be periodic with period $P$, you need to check that the pairs of qubits $x_0$ and $x_P$, $x_1$ and $x_{P + 1}$, and so on are the same. 

We can use a similar approach to the solution as well: do the comparisons in-place using $CNOT$ gates. 

If you iterate through the pairs in order from left to right, starting with the pair $x_0$ and $x_P$, you'll need to make sure to store the XORs in the left qubit of the pair. When comparing the states of qubits $x_j$ and $x_{j + P}$, the right qubit $x_{j + P}$ might be involved in another comparison later, if the position $j + 2P$ is within the bit string, so you shouldn't modify its state in the earlier comparison. However, the qubit $x_j$ was already compared with the qubit $x_{j-P}$, so it's safe to modify its state now.

Same as in the previous task, we then check that all XORs are $0$ using controlled-on-zero $X$ gate (with the first $N-P$ qubits as controls), and finally uncompute any changes we did to the input register.

@[solution]({
    "id": "marking_oracles__periodic_p_solution",
    "codePath": "./Solution.qs"
})
