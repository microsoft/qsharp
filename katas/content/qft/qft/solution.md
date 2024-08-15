Let's use the hint and start by preparing the described state with the qubits reversed:

$$\begin{align*}
&\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_1 j_2 ... j_{n-1} j_n} \ket{1} \big) \otimes \\
&\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_2 ... j_{n-1} j_n} \ket{1} \big) \otimes ... \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_{n-1} j_n} \ket{1} \big) \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_n} \ket{1} \big)
\end{align*}$$

You've already found a way to prepare a binary fraction exponent (the desired state of the first qubit) in-place in the previous task. The state $\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_1 j_2 ... j_{n-1} j_n} \ket{1} \big) \otimes \ket{j_2 ... j_n}$ can be prepared by first applying the Hadamard gate to the first qubit in state $\ket{j_1}$ and then applying a succession of controlled rotations using qubits $\ket{j_k}$ with increasing values of $k$ as controls, so that an extra phase terms from $e^{2\pi i \cdot j_2/2^2}$ all the way up to $e^{2\pi i \cdot j_n/2^{n}}$ are added with each rotation. 

This will prepare the first qubit in the right state. You can see that $j_1$ doesn't appear in the rest of the expression for the target state, so you won't need to use it in the rest of the code.

To prepare the remaining qubits in the right states, you can work backwards from this state.
The second qubit in the sequence needs to be prepared in the state $\frac{1}{\sqrt{2}} \big(\ket{0} + e^{2\pi i \cdot 0.j_2 ... j_{n-1} j_n} \ket{1} \big)$. 
Similarly to the first qubit, you can do this by applying a Hadamard gate to the qubit $\ket{j_2}$ and then using qubits $\ket{j_3}$ to $\ket{j_n}$ to apply $n-2$ controlled rotation gates to the second qubit. 
After this operation, the total system state will be: 

$$\frac{1}{\sqrt{2}} \big(\ket{0} + e^{2\pi i \cdot 0.j_1 j_2 ... j_{n-1} j_n} \ket{1} \big)\otimes
\frac{1}{\sqrt{2}} \big(\ket{0} + e^{2\pi i \cdot 0.j_2 ... j_{n-1} j_n} \ket{1} \big) \otimes \ket{j_3 ... j_n}$$

These two steps allow us to see a pattern: for each qubit $j_k, k = 1 .. n$ :

1. Apply a Hadamard gate to $|j_k\rangle$.
2. Apply the controlled rotation operator $n-k$ times to the qubit $|j_k\rangle$, using qubits $|j_{k+1}\rangle$ through $|j_n\rangle$ as the controls, with phases corresponding to fractions from $2^2$ to $2^{n-k+1}$.

The effect of these steps will be preparing the state that almost matches the required state, but has the qubit order reversed compared to it. 
You can fix that by using a series of $\textrm{SWAP}$ gates to reverse the order of qubits in the state.

@[solution]({
"id": "qft__qft_solution",
"codePath": "./Solution.qs"
})
