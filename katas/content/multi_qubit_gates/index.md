# Multi-Qubit Gates

@[section]({
    "id": "multi_qubit_gates_overview",
    "title": "Overview"
})

This kata continues the introduction to quantum gates, focusing on applying quantum gates to multi-qubit systems.

It covers the following topics:

- Applying quantum gates to a part of the system
- `CNOT` and `SWAP` gates
- Controlled gates

@[section]({
    "id": "multi_qubit_gates_overview",
    "title": "The Basics"
})

As a reminder, single-qubit gates are represented by $2\\times2$ unitary matrices.

The effect of a gate applied to a qubit can be calculated by multiplying the corresponding matrix by the state vector of the qubit to get the resulting state vector.

Multi-qubit gates are represented by $2^N\\times2^N$ matrices, where $N$ is the number of qubits the gate operates on. To apply this gate, you multiply the matrix by the state vector of the $N$-qubit quantum system.

## Applying Gates to a Part of the System

The simplest thing we can do with multi-qubit systems is to apply gates to only a subset of qubits in the system.
Similar to how it is sometimes possible to represent the state of a multi-qubit systems as a tensor product of single-qubit states, you can construct gates that modify the state of a multi-qubit system as tensor products of gates that affect parts of the system.

Let's consider an example of applying single-qubit gates to one of the qubits of a two-qubit system.
If you want to apply an $X$ gate to the first qubit of the system and do nothing to the second qubit, the resulting gate will be represented as a tensor product of an $X$ gate and the identity gate $I$ which corresponds to doing nothing:

$$
X \otimes I =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix} \otimes \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix} =
\begin{bmatrix}
    0 & 0 & 1 & 0 \\
    0 & 0 & 0 & 1 \\
    1 & 0 & 0 & 0 \\
    0 & 1 & 0 & 0
\end{bmatrix}
$$

You can use the same approach when applying several gates to independent parts of the system at the same time.
For example, applying the $X$ gate to the first qubit and the $H$ gate to the second qubit would be represented as follows:

$$
X \otimes H =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix} \otimes \frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix} =
\frac{1}{\sqrt{2}}\begin{bmatrix}
    0 & 0 & 1 & 1 \\
    0 & 0 & 1 & -1 \\
    1 & 1 & 0 & 0 \\
    1 & -1 & 0 & 0
\end{bmatrix}
$$

> Note that we can use mixed-multiplication property of tensor product to see that this is equivalent to applying $X$ gate to the first qubit and applying $H$ gate to the second qubit, in either order:\n",
>
> $$X \otimes H = (I X) \otimes (H I) = (I \otimes H) (X \otimes I)$$
> $$X \otimes H = (X I) \otimes (I H) = (X \otimes I) (I \otimes H)$$

This approach can be generalized to larger systems and gates that act on multiple qubits as well.
It can be less straightforward if a multi-qubit gate is applied to a subset of qubits that are not "adjacent" to each other in the tensor product; we'll see an example later in this tutorial."

@[exercise]({
    "id": "preparing_bell_state",
    "title": "Preparing a Bell state",
    "descriptionPath": "./preparing_bell_state/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./preparing_bell_state/Verification.qs"
    ],
    "placeholderSourcePath": "./preparing_bell_state/Placeholder.qs",
    "solutionPath": "./preparing_bell_state/solution.md"
})
