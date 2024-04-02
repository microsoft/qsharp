The single-qubit GHZ state is the plus state $\frac{1}{\sqrt{2}} \big (|0\rangle + |1\rangle\big)$ that we've discussed in the first task. As a reminder, that state is prepared by applying a Hadamard gate.

The 2-qubit GHZ state is the Bell state $\frac{1}{\sqrt{2}} \big (|00\rangle + |11\rangle\big)$ that we've discussed in the two previous tasks.

The next one is the 3-qubit GHZ state:
$$|GHZ\rangle = \frac{1}{\sqrt{2}} \big (|000\rangle + |111\rangle\big)$$

Let's use the 2-qubit state as a building block to construct the state for 3 qubits. First, let's add a third qubit to the above state (on the right from the first two qubits).
Comparing this state with the desired end state, we see that they differ only in the third (rightmost) qubit:

$$|\Phi^+\rangle |0\rangle = \frac{1}{\sqrt{2}} \big (|000\rangle + |11\textbf{0}\rangle\big)$$
$$|GHZ\rangle = \frac{1}{\sqrt{2}} \big (|000\rangle + |11\textbf{1}\rangle\big)$$

Applying a controlled NOT operation using the first (leftmost) qubit as the control bit and the third (rightmost) qubit as the target qubit allows us to fix this difference.

Thus we can come to the general solution: apply Hadamard gate to the first qubit and do a series of CNOT gates with the first qubit as control and each of the other qubits as targets.

@[solution]({
    "id": "superposition__ghz_solution",
    "codePath": "./Solution.qs"
})
