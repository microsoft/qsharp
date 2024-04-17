Firstly we notice that we are dealing with an unentangled pair of qubits. In vector form this is:
In vector form the transformation we need is

In vector form the transformation we need is 
$$
\frac{1}{2}\begin{bmatrix}1\\\ 1\\\ 1\\\ 1\\\ \end{bmatrix} 
\rightarrow 
\frac{1}{2}\begin{bmatrix}1\\\ 1\\\ 1\\\ -1\\\ \end{bmatrix}
$$

All that needs to happen to change the input into the goal is that the $|11\rangle$ basis state needs to have its sign flipped.

We remember that the Pauli Z gate flips signs in the single qubit case, so we need to investigate if there is a 2-qubit version of this gate that we can use here. We can also recall task "Phase i" which dealt with phase shifts and, remembering that $e^{i\cdot\pi} = -1$, we can think of the transformation we're looking for as a phase shift.
It can be useful to investigate a general case and then use it to perform a specific state change, so let's look for a 2-qubit variant of the phase shift.
Similarly to task "Two qubit gate 1", the phase shift only occurs on one of the basis states, so this suggests it might be a conditional shift. If we could have our phase shift applied to `qs[1]` conditional on `qs[0]` being in the state $|1\rangle$, then we would have a description of our gate. If we now look through a list of gates in the [Single-qubit gates tutorial](../tutorials/SingleQubitGates/SingleQubitGates.ipynb), we'll find the R1 phase shift gate with angle parameter $\theta$ (radians), defined as

$$
R1(\alpha)= 
 \begin{bmatrix}1 & 0\\\ 0 & \color{red}{e^{i\alpha}} \end{bmatrix}
$$

The controlled variant of this gate will look like this:

$$
CR1(\alpha) = 
 \begin{bmatrix}1 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 0 & 0 &\color{red}{e^{i\alpha}} \end{bmatrix}
$$

This gate is almost Pauli I, the identity gate, with the different in just the last column, showing what will happen to the $|11\rangle$ basis state. Applying it to our input state for $\alpha = \pi$, we'll get:

$$
\frac{1}{2}\begin{bmatrix}1 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 0 & 0 &\color{red}{e^{i\alpha}} \end{bmatrix}
\begin{bmatrix}1\\\ 1\\\ 1\\\ 1\\\ \end{bmatrix}=
\frac{1}{2}\begin{bmatrix}1\\\ 1\\\ 1\\\ 1\cdot\color{red}{e^{i\alpha}}\\ \end{bmatrix}=
\frac{1}{2} \big( |00\rangle + |01\rangle + |10\rangle {\color{red}-} |11\rangle \big)
$$

The last thing we notice if we look through the [list of operations in the Microsoft.Quantum.Canon namespace](https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.canon) is the CZ (Controlled Z) gate, a special case of CR1 that implements exactly this gate.

@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_2_solution_a",
"codePath": "./SolutionA.qs"
})
Alternatively, we can express this gate using the intrinsic gate Z and its controlled variant using the Controlled functor:

@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_2_solution_b",
"codePath": "./SolutionB.qs"
})