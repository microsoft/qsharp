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
\begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix} \otimes \begin{bmatrix} 1 & 0 \\\ 0 & 1 \end{bmatrix} =
\begin{bmatrix}
    0 & 0 & 1 & 0 \\\ 
    0 & 0 & 0 & 1 \\\ 
    1 & 0 & 0 & 0 \\\ 
    0 & 1 & 0 & 0
\end{bmatrix}
$$

You can use the same approach when applying several gates to independent parts of the system at the same time.
For example, applying the $X$ gate to the first qubit and the $H$ gate to the second qubit would be represented as follows:

$$
X \otimes H =
\begin{bmatrix} 0 & 1 \\\ 1 & 0 \end{bmatrix} \otimes \frac{1}{\sqrt{2}}\begin{bmatrix} 1 & 1 \\\ 1 & -1 \end{bmatrix} =
\frac{1}{\sqrt{2}}\begin{bmatrix}
    0 & 0 & 1 & 1 \\\ 
    0 & 0 & 1 & -1 \\\ 
    1 & 1 & 0 & 0 \\\ 
    1 & -1 & 0 & 0
\end{bmatrix}
$$

> Note that we can use mixed-multiplication property of tensor product to see that this is equivalent to applying $X$ gate to the first qubit and applying $H$ gate to the second qubit, in either order:
>
> $$X \otimes H = (I X) \otimes (H I) = (I \otimes H) (X \otimes I)$$
> $$X \otimes H = (X I) \otimes (I H) = (X \otimes I) (I \otimes H)$$

This approach can be generalized to larger systems and gates that act on multiple qubits as well.
It can be less straightforward if a multi-qubit gate is applied to a subset of qubits that are not "adjacent" to each other in the tensor product; we'll see an example later in this kata.

@[exercise]({
    "id": "compound_gate",
    "title": "Compound Gate",
    "descriptionPath": "./compound_gate/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./compound_gate/Verification.qs"
    ],
    "placeholderSourcePath": "./compound_gate/Placeholder.qs",
    "solutionPath": "./compound_gate/solution.md"
})

@[section]({
    "id": "multi_qubit_gates_cnot_gate",
    "title": "CNOT Gate"
})

Our first proper multi-qubit gate is the `CNOT` ("controlled NOT") gate.
The `CNOT` gate is a two-qubit gate, the first qubit is referred to as the **control** qubit, and the second as the **target** qubit.
`CNOT` acts as a conditional gate of sorts: if the control qubit is in state $|1\\rangle$, it applies the `X` gate to the target qubit, otherwise it does nothing.

> If the system is in a superposition of several basis states, the effects of the gate will be a linear combination of the effects of it acting separately on each of the basis states.
> This will be the case for all quantum gates you'll encounter later that are specified in terms of basis states: since all unitary gates are linear, it is sufficient to define their effect on the basis states, and use linearity to figure out their effect on any state.

<table>
    <tr>
        <th>Gate</th>
        <th>Matrix</th>
        <th>Applying to $|\psi\rangle = \alpha|00\rangle + \beta|01\rangle + \gamma|10\rangle + \delta|11\rangle$</th>
        <th>Applying to basis states</th>
        <th>Q# Documentation</th>
    </tr>
    <tr>
        <td>$\text{CNOT}</td>
        <td>$\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 1 \\\ 0 & 0 & 1 & 0 \end{bmatrix}$</td>
        <td>$\text{CNOT}|\psi\rangle = \alpha|00\rangle + \beta|01\rangle + \delta|10\rangle + \gamma|11\rangle$</td>
        <td>
            $$\text{CNOT}|00\rangle = |00\rangle$$
            $$\text{CNOT}|01\rangle = |01\rangle$$
            $$\text{CNOT}|10\rangle = |11\rangle$$
            $$\text{CNOT}|11\rangle = |10\rangle$$
        </td>
        <td><a href=\"https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.cnot\">CNOT</a></td>
    </tr>
</table>

The `CNOT` gate is particularly useful for preparing entangled states. Consider the following separable state:

$$\big(\alpha|0\rangle + \beta|1\rangle\big) \otimes |0\rangle = \alpha|00\rangle + \beta|10\rangle$$

If we apply the $\\text{CNOT}$ gate to it, with the first qubit as the control, and the second as the target, we get the following state, which is not separable any longer:

$$\alpha|00\rangle + \beta|11\rangle$$

The `CNOT` gate is self-adjoint: applying it for the second time reverses its effect.

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

@[section]({
    "id": "multi_qubit_gates_ket_bra_representation",
    "title": "Ket-bra Representation"
})

Same as in the case of single-qubit gates, we can represent multi-qubit gates using Dirac notation.

> Recall that kets represent column vectors and bras represent row vectors. For any ket $|\psi\rangle$, the corresponding bra is its adjoint (conjugate transpose): $\langle\psi| = |\psi\rangle^\dagger$.
>
> Kets and bras are used to express inner and outer products. The inner product of $|\phi\rangle$ and $|\psi\rangle$ is the matrix product of $\langle\phi|$ and $|\psi\rangle$, denoted as $\langle\phi|\psi\rangle$, and their outer product is the matrix product of $|\phi\rangle$ and $\langle\psi|$, denoted as $|\phi\rangle\langle\psi|$.
>
> As we've seen in the single-qubit gates tutorial, kets and bras can be used to represent matrices. The outer product of two vectors of the same size produces a square matrix. We can use a linear combination of several outer products of simple vectors (such as basis vectors) to express any square matrix.

Let's consider ket-bra representation of the $\\text{CNOT}$ gate:

$$\text{CNOT} =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + |10\rangle\langle11| + |11\rangle\langle10| =$$
$$
\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix}\begin{bmatrix} 1 & 0 & 0 & 0 \end{bmatrix} +
\begin{bmatrix} 0 \\\ 1 \\\ 0 \\\ 0 \end{bmatrix}\begin{bmatrix} 0 & 1 & 0 & 0 \end{bmatrix} +
\begin{bmatrix} 0 \\\ 0 \\\ 1 \\\ 0 \end{bmatrix}\begin{bmatrix} 0 & 0 & 0 & 1 \end{bmatrix} +
\begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}\begin{bmatrix} 0 & 0 & 1 & 0 \end{bmatrix} =
$$ 
$$
\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ \end{bmatrix} + 
\begin{bmatrix} 0 & 0 & 0 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ \end{bmatrix} + 
\begin{bmatrix} 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 1 \\\ 0 & 0 & 0 & 0 \\\ \end{bmatrix} + 
\begin{bmatrix} 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 0 & 0 \\\ 0 & 0 & 1 & 0 \\\ \end{bmatrix} =
$$
$$\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 1 \\\ 0 & 0 & 1 & 0 \\\ \end{bmatrix}$$

This representation can be used to carry out calculations in Dirac notation without ever switching back to matrix representation:

$$
\text{CNOT}|10\rangle = 
\big(|00\rangle\langle00| + |01\rangle\langle01| + |10\rangle\langle11| + |11\rangle\langle10|\big)|10\rangle =$$
$$|00\rangle\langle00|10\rangle + |01\rangle\langle01|10\rangle + |10\rangle\langle11|10\rangle + |11\rangle\langle10|10\rangle =$$
$$|00\rangle\big(\langle00|10\rangle\big) + |01\rangle\big(\langle01|10\rangle\big) + |10\rangle\big(\langle11|10\rangle\big) + |11\rangle\big(\langle10|10\rangle\big) =$$
$$|00\rangle(0) + |01\rangle(0) + |10\rangle(0) + |11\rangle(1) = |11\rangle$$

> Notice how a lot of the inner product terms turn out to equal 0, and our expression is easily simplified. We have expressed the CNOT gate in terms of outer product of computational basis states, which are orthonormal, and apply it to another computational basis state, so the individual inner products are going to always be 0 or 1.

In general case, a $4\\times4$ matrix that describes a 2-qubit gate
$$A =
\begin{bmatrix}
    a_{00} & a_{01} & a_{02} & a_{03} \\\ 
    a_{10} & a_{11} & a_{12} & a_{13} \\\ 
    a_{20} & a_{21} & a_{22} & a_{23} \\\ 
    a_{30} & a_{31} & a_{32} & a_{33} \\\ 
\end{bmatrix}
$$

will have the following ket-bra representation:
$$A =$$
$$a_{00} |00\rangle\langle00| + a_{01} |00\rangle\langle01| + a_{02} |00\rangle\langle10| + a_{03} |00\rangle\langle11| +$$
$$a_{10} |01\rangle\langle00| + a_{11} |01\rangle\langle01| + a_{12} |01\rangle\langle10| + a_{13} |01\rangle\langle11| +$$
$$a_{20} |10\rangle\langle00| + a_{21} |10\rangle\langle01| + a_{22} |10\rangle\langle10| + a_{23} |10\rangle\langle11| +$$
$$a_{30} |11\rangle\langle00| + a_{31} |11\rangle\langle01| + a_{32} |11\rangle\langle10| + a_{33} |11\rangle\langle11|$$

A similar expression can be extended for matrices that describe $N$-qubit gates, where $N > 2$:

$$A = \sum_{i=0}^{2^N-1} \sum_{j=0}^{2^N-1} a_{ij} |i\rangle\langle j|$$

Dirac notation is particularly useful for expressing sparse matrices - matrices that have few non-zero elements. Indeed, consider the `CNOT` gate again: it is a $4 \times 4$ matrix described with 16 elements, but its Dirac notation has only 4 terms, one for each non-zero element of the matrix.

With enough practice you'll be able to perform computations in Dirac notation without spelling out all the bra-ket terms explicitly!

@[section]({
    "id": "multi_qubit_gates_ket_bra_decomposition",
    "title": "Ket-bra Decomposition"
})

This section describes a more formal process of finding the ket-bra decompositions of multi-qubit quantum gates.
This section is not necessary to start working with quantum gates, so feel free to skip it for now, and come back to it later.

You can use the properties of eigenvalues and eigenvectors to find the ket-bra decomposition of any gate. Consider an $N$-qubit gate $A$; the matrix representation of the gate is a square matrix of size $2^N$. Therefore it also has $2^N$ orthogonal eigenvectors $|\psi_i\rangle$

$$A|\psi_i\rangle = x_i|\psi_i\rangle, 0 \leq i \leq 2^N -1$$

Then its ket-bra decomposition is:

$$A = \sum_{i=0}^{2^N-1} x_i|\psi_i\rangle\langle\psi_i|$$

Let's use our `CNOT` gate as a simple example.
The $\\text{CNOT}$ gate has four eigenvectors.
 * Two, as we can clearly see, are computational basis states $|00\rangle$ and $|01\rangle$ with eigen values $1$ and $1$, respectively (the basis states that are not affected by the gate).
 * The other two are $|1\rangle \otimes |+\rangle = \frac{1}{\sqrt{2}}\big(|10\rangle + |11\rangle\big)$ and $|1\rangle \otimes |-\rangle = \frac{1}{\sqrt{2}}\big(|10\rangle - |11\rangle\big)$ with eigenvalues $1$ and $-1$, respectively:

$$\text{CNOT}|0\rangle \otimes |0\rangle = |0\rangle \otimes |0\rangle$$
$$\text{CNOT}|0\rangle \otimes |1\rangle = |0\rangle \otimes |1\rangle$$
$$\text{CNOT}|1\rangle \otimes |+\rangle = |1\rangle \otimes |+\rangle$$
$$\text{CNOT}|1\rangle \otimes |-\rangle = -|1\rangle \otimes |-\rangle$$

Here's what the decomposition looks like:

$$\text{CNOT} =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + 1\rangle \otimes |+\rangle\langle1| \otimes \langle +| - |1\rangle \otimes| -\rangle\langle1| \otimes \langle -| =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + \frac{1}{2}\big[\big(|10\rangle + |11\rangle\big)\big(\langle10| + \langle11|\big) - \big(|10\rangle - |11\rangle\big)\big(\langle10| - \langle11|\big)\big] =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + \frac{1}{2}\big(|10\rangle\langle10| + |10\rangle\langle11| + |11\rangle\langle10| + |11\rangle\langle11| - |10\rangle\langle10| + |10\rangle\langle11| + |11\rangle\langle10| - |11\rangle\langle11|\big) =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + \frac{1}{2}\big(2|10\rangle\langle11| + 2|11\rangle\langle10|\big) =$$
$$|00\rangle\langle00| + |01\rangle\langle01| + |10\rangle\langle11| + |11\rangle\langle10|$$"
