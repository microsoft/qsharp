# Multi-Qubit Gates

@[section]({
    "id": "multi_qubit_gates_overview",
    "title": "Overview"
})

This Kata continues the introduction to quantum gates, focusing on applying quantum gates to multi-qubit systems.

**This Kata covers the following topics:**
- Applying quantum gates to a part of the system
- `CNOT` and `SWAP` gates
- Controlled gates

**What you should know to start working on this Kata:**
- Basic linear algebra
- The concept of qubit and multi-qubit systems
- Single-qubit and multi-qubit quantum gates

@[section]({
    "id": "multi_qubit_gates_the_basics",
    "title": "The Basics"
})

As a reminder, single-qubit gates are represented by $2\times2$ unitary matrices.

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
It is more complex when a multi-qubit gate is applied to a subset of qubits that are not "adjacent" to each other in the tensor product; we'll see an example later in this Kata.

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
        <td>$\text{CNOT}$</td>
        <td>$\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 1 \\\ 0 & 0 & 1 & 0 \end{bmatrix}$</td>
        <td>$\text{CNOT}|\psi\rangle = \alpha|00\rangle + \beta|01\rangle + \delta|10\rangle + \gamma|11\rangle$</td>
        <td>
            $$\text{CNOT}|00\rangle = |00\rangle$$
            $$\text{CNOT}|01\rangle = |01\rangle$$
            $$\text{CNOT}|10\rangle = |11\rangle$$
            $$\text{CNOT}|11\rangle = |10\rangle$$
        </td>
        <td><a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.cnot">CNOT</a></td>
    </tr>
</table>

The `CNOT` gate is particularly useful for preparing entangled states. Consider the following separable state:

$$\big(\alpha|0\rangle + \beta|1\rangle\big) \otimes |0\rangle = \alpha|00\rangle + \beta|10\rangle$$

If we apply the $\\text{CNOT}$ gate to it, with the first qubit as the control, and the second as the target, we get the following state, which is not separable any longer:

$$\alpha|00\rangle + \beta|11\rangle$$

The `CNOT` gate is self-adjoint: applying it for the second time reverses its effect.

@[exercise]({
    "id": "preparing_bell_state",
    "title": "Preparing a Bell State",
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
    "title": "Ket-Bra Representation"
})

Same as in the case of single-qubit gates, we can represent multi-qubit gates using Dirac notation.

> Recall that kets represent column vectors and bras represent row vectors. For any ket $|\psi\rangle$, the corresponding bra is its adjoint (conjugate transpose): $\langle\psi| = |\psi\rangle^\dagger$.
>
> Kets and bras are used to express inner and outer products. The inner product of $|\phi\rangle$ and $|\psi\rangle$ is the matrix product of $\langle\phi|$ and $|\psi\rangle$, denoted as $\langle\phi|\psi\rangle$, and their outer product is the matrix product of $|\phi\rangle$ and $\langle\psi|$, denoted as $|\phi\rangle\langle\psi|$.
>
> As we've seen in the "Single-Qubit Gates" Kata, kets and bras can be used to represent matrices. The outer product of two vectors of the same size produces a square matrix. We can use a linear combination of several outer products of simple vectors (such as basis vectors) to express any square matrix.

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
    "title": "Ket-Bra Decomposition"
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
$$|00\rangle\langle00| + |01\rangle\langle01| + |10\rangle\langle11| + |11\rangle\langle10|$$

@[section]({
    "id": "multi_qubit_gates_swap_gate",
    "title": "SWAP Gate"
})

The `SWAP` gate acts on two qubits, and, as the name implies, swaps their quantum states.

<table>
    <tr>
        <th>Gate</th>
        <th>Matrix</th>
        <th>Applying to $|\psi\rangle = \alpha|00\rangle + \beta|01\rangle + \gamma|10\rangle + \delta|11\rangle$</th>
        <th>Applying to basis states</th>
        <th>Q# Documentation</th>
    </tr>
    <tr>
        <td>$\text{SWAP}$</td>
        <td>$\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 0 & 1 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 1 \end{bmatrix}$</td>
        <td>$\text{SWAP}|\psi\rangle = \alpha|00\rangle + \gamma|01\rangle + \beta|10\rangle + \delta|11\rangle$</td>
        <td>
            $$\text{SWAP}|00\rangle = |00\rangle$$
            $$\text{SWAP}|01\rangle = |10\rangle$$
            $$\text{SWAP}|10\rangle = |01\rangle$$
            $$\text{SWAP}|11\rangle = |11\rangle$$
        <td><a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.swap">SWAP</a></td>
    </tr>
</table>

@[exercise]({
    "id": "qubit_swap",
    "title": "Qubit SWAP",
    "descriptionPath": "./qubit_swap/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./qubit_swap/Verification.qs"
    ],
    "placeholderSourcePath": "./qubit_swap/Placeholder.qs",
    "solutionPath": "./qubit_swap/solution.md"
})

@[section]({
    "id": "multi_qubit_gates_acting_on_non_adjacent_qubits",
    "title": "Multi-Qubit Gates Acting on Non-Adjacent Qubits"
})

In the above examples the `CNOT` gate acted on two adjacent qubits. However, multi-qubit gates can act on non-adjacent qubits as well. Let's see how to work out the math of the system state change in this case.

Take 3 qubits in an arbitrary state $|\psi\rangle = x_{000} |000\rangle + x_{001}|001\rangle + x_{010}|010\rangle + x_{011}|011\rangle + x_{100}|100\rangle + x_{101}|101\rangle + x_{110}|110\rangle + x_{111}|111\rangle $.

We can apply the `CNOT` gate on 1st and 3rd qubits, with the 1st qubit as control and the 3rd qubit as target. Let's label the 3-qubit gate that describes the effect of this on the whole system as `CNOT`. The `CINOT` ignores the 2nd qubit (leaves it unchanged) and applies the `CNOT` gate as specified above.

## Q#

In Q# we describe the operation as the sequence of gates that are applied to the qubits, regardless of whether the qubits are adjacent or not.

```qsharp
operation CINOT (qs: Qubit[]) : Unit {
    CNOT(qs[0], qs[2]); // Length of qs is assumed to be 3
}
```

## Dirac Notation

In Dirac notation we can consider the effect of the gate on each basis vector separately: each basis vector $|a_1a_2a_3\rangle$ remains unchanged if $a_1 = 0$, and becomes $|a_1a_2(\neg a_3)\rangle$ if $a_1 = 1$. The full effect on the state becomes:

$$\text{CINOT}|\psi\rangle = x_{000} \text{CINOT}|000\rangle + x_{001} \text{CINOT}|001\rangle + x_{010} \text{CINOT}|010\rangle + x_{011} \text{CINOT}|011\rangle+$$
$$x_{100} \text{CINOT}|100\rangle + x_{101} \text{CINOT}|101\rangle + x_{110} \text{CINOT}|110\rangle + x_{111} \text{CINOT}|111\rangle =$$
$$x_{000}|000\rangle + x_{001}|001\rangle + x_{010}|010\rangle + x_{011}|011\rangle + x_{101}|100\rangle + x_{100}|101\rangle + x_{111}|110\rangle + x_{110}|111\rangle $$

## Matrix Form

$\text{CINOT}$ can also be represented in matrix form as a $2^3 \times 2^3$ matrix:
$$
\begin{bmatrix}
    1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
\end{bmatrix}
$$

Applying $\text{CINOT}$ to $|\psi\rangle$ gives us
$$
\text{CINOT} \begin{bmatrix}
    1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
\end{bmatrix}
\begin{bmatrix}
    x_{000} \\\ x_{001} \\\ x_{010} \\\ x_{011} \\\ x_{100} \\\ x_{101} \\\ x_{110} \\\ x_{111}
\end{bmatrix} =
\begin{bmatrix}
    x_{000} \\\ x_{001} \\\ x_{010} \\\ x_{011} \\\ x_{101} \\\ x_{100} \\\ x_{111} \\\ x_{110}
\end{bmatrix}
$$

However, as $N$ gets larger, creating a full size matrix can be extremely unwieldy. To express the matrix without spelling out its elements, we can use the following trick:

1. Apply the `SWAP` gate on the 1st and 2nd qubits.
   This will bring the qubits on which the `CNOT` gate acts next to each other, without any extra qubits between them.
2. Apply the `CNOT` on 2nd and 3rd qubits.
   Since now the gate acts on adjacent qubits, this can be represented as a tensor product of the gate we're applying and `I` gates.
3. Apply the `SWAP` gate on the 1st and 2nd qubits again.

These can be represented as applying the following gates on the 3 qubits.

1. $\text{SWAP} \otimes I$
$$
x_{000}|000\rangle + x_{001}|001\rangle + x_{100}|010\rangle + x_{101}|011\rangle +
x_{010}|100\rangle + x_{011}|101\rangle + x_{110}|110\rangle + x_{111}|111\rangle
$$

2. $I \otimes \text{CNOT}$
$$
x_{000}|000\rangle + x_{001}|001\rangle + x_{101}|010\rangle + x_{100}|011\rangle +
x_{010}|100\\rangle + x_{011}|101\rangle + x_{111}|110\rangle + x_{110}|111\rangle
$$

3. $\text{SWAP} \otimes I$
$$
x_{000}|000\rangle + x_{001}|001\rangle + x_{010}|010\rangle + x_{011}|011\rangle +
x_{101}|100\rangle + x_{100}|101\rangle + x_{111}|110\rangle + x_{110}|111\rangle
$$

The result is the the $\text{CINOT}$ gate as we intended; so we can write

$$\text{CINOT} = (\text{SWAP} \otimes I)(I \otimes \text{CNOT})(\text{SWAP} \otimes I)$$

> Note that in matrix notation we always apply a gate to the complete system, so we must apply $\text{SWAP} \otimes I$, spelling the identity gate explicitly.
> However, when implementing the unitary $\text{SWAP} \otimes I$ in Q#, we need only to call `SWAP(qs[0], qs[1])` - the remaining qubit `qs[2]` will not change, which is equivalent to applying an implicit identity gate.
>
> We can also spell out all gates applied explicitly (this makes for a much longer code, though):
> ```qsharp
operation CINOT (qs: Qubit[]) : Unit {
    // First step
    SWAP(qs[0], qs[1]);
    I(qs[2]);
    // Second step
    I(qs[0]);
    CNOT(qs[1], qs[2]);
    // Third step
    SWAP(qs[0], qs[1]);
    I(qs[2]);
}
```

@[section]({
    "id": "multi_qubit_gates_controlled_gates",
    "title": "Controlled Gates"
})

**Controlled gates** are a class of gates derived from other gates as follows: they act on a control qubit and a target qubit, just like the `CNOT` gate.
A controlled-`U` gate applies the `U` gate to the target qubit if the control qubit is in state $|1\rangle$, and does nothing otherwise.

Given a gate $U = \begin{bmatrix} \alpha & \beta \\\ \gamma & \delta \end{bmatrix}$, its controlled version looks like this:

<table>
    <tr>
        <th>Gate</th>
        <th>Matrix</th>
        <th>Q# Documentation</th>
    </tr>
    <tr>
        <td>$\text{Controlled U}$</td>
        <td>
            $$
            \begin{bmatrix}
                1 & 0 & 0 & 0 \\\ 
                0 & 1 & 0 & 0 \\\ 
                0 & 0 & \alpha & \beta \\\ 
                0 & 0 & \gamma & \delta
            \end{bmatrix}
            $$
        </td>
        <td><a href="https://docs.microsoft.com/azure/quantum/user-guide/language/expressions/functorapplication#controlled-functor">Controlled functor</a></td>
    </tr>
</table>

> The CNOT gate is en example of a controlled gate, which is why it is also known as the controlled NOT or controlled `X` gate.

The concept of controlled gates can be generalized beyond controlling single-qubit gates.
For any multi-qubit gate, its controlled version will have an identity matrix in the top left quadrant, the gate itself in the bottom right, and $0$ everywhere else.
Here, for example, is the `Controlled SWAP`, or **Fredkin gate**:

$$
\begin{bmatrix}
    1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 1 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 0 & 1
\end{bmatrix}
$$

In Q#, controlled gates are applied using the [`Controlled`](https://docs.microsoft.com/azure/quantum/user-guide/language/expressions/functorapplication#controlled-functor) functor.
The controlled version of a gate accepts an array of control qubits (in this case an array of a single qubit), followed by the arguments to the original gate.
For example, these two lines are equivalent:

```qsharp
Controlled X([control], target);
CNOT(control, target);
```

If the original gate was implemented as an operation with multiple parameters, the controlled version of this gate will take those parameters as a tuple. For example, to apply Fredkin gate, you'd have to call:

```qsharp
Controlled SWAP([control], (q1, q2));
```

You can use the controlled version of a Q# operation only if that operation has a controlled version defined.
The Q# compiler will often be able to generate a controlled version of the operation automatically if you put `is Ctl` after the operation's return type.
In other cases, you'll need to define the controlled version of an operation manually.

@[exercise]({
    "id": "controlled_rotation",
    "title": "Controlled Rotation",
    "descriptionPath": "./controlled_rotation/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./controlled_rotation/Verification.qs"
    ],
    "placeholderSourcePath": "./controlled_rotation/Placeholder.qs",
    "solutionPath": "./controlled_rotation/solution.md"
})

@[section]({
    "id": "multi_qubit_gates_multi_controlled_gates",
    "title": "Multi-Controlled Gates"
})

Controlled gates can have multiple control qubits; in this case the gate $U$ is applied only if all control qubits are in the $|1\rangle$ states.
You can think of it as constructing a controlled version of a gate that is already controlled.

The simplest example of this is the **Toffoli gate**, or `CCNOT` (controlled controlled `NOT`) gate, which applies the `X` gate to the last qubit only if the first two qubits are in $|11\rangle$ state:

$$
\begin{bmatrix}
    1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\\ 
    0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
\end{bmatrix}
$$

To construct a multi-controlled version of an operation in Q#, you can use the Controlled functor as well, passing all control qubits as an array that is the first parameter.

@[section]({
    "id": "multi_qubit_gates_other_controlled_gates",
    "title": "Other Types of Controlled Gates"
})

Typically the term "controlled `U` gate" refers to the type of gate we've described previously, which applies the gate `U` only if the control qubit(s) are in the $|1\rangle$ state.

It is possible, however, to define variants of controlled gates that use different states as control states.
For example, an **anti-controlled** `U` gate (sometimes called **zero-controlled**) applies a gate only if the control qubit is in the $|0\rangle$ state.
It is also possible to define control conditions in other bases, for example, applying the gate if the control qubit is in the $|+\rangle$ state.

All the variants of controlled gates can be expressed in terms of the controls described in previous sections, using the following sequence of steps:
* First, apply a transformation on control qubits that will transform the state you want to use as control into the $|1...1\rangle$ state.
* Apply the regular controlled version of the gate.
* Finally, undo the transformation on control qubits from the first step using the adjoint version of it.

> Why do we need this last step? Remember that controlled gates are defined in terms of their effect on the basis states:
> we apply the gate on the target qubit if and only if the control qubit is in the state we want to control on, and we don't change the state of the control qubit at all.
> If we don't undo the transformation we did on the first step, applying our gate to a basis state will modify not only the state of the target qubit but also the state of the control qubit, which is not what we're looking for.
>
> For example, consider an anti-controlled `X` gate - a gate that should apply an $X$ gate to the second qubit if the first qubit is in the $|0\rangle$ state.
> Here is the effect we expect this gate to have on each of the 2-qubit basis states:
>
> <table>
  <tr>
    <th>Input state</th>
    <th>Output state</th>
  </tr>
  <tr>
    <td>$|00\rangle$</td>
    <td>$|01\rangle$</td>
  </tr>
  <tr>
    <td>$|01\rangle$</td>
    <td>$|00\rangle$</td>
  </tr>
  <tr>
    <td>$|10\rangle$</td>
    <td>$|10\rangle$</td>
  </tr>
  <tr>
    <td>$|11\rangle$</td>
    <td>$|11\rangle$</td>
  </tr>
</table>

> Let's apply the anti-controlled `X` gate to the $|00\rangle$ state step by step:
> 1. Transform the state of the control qubit to $|1\rangle$: we can do that by applying the $X$ gate to the first qubit:
> $$|00\rangle \rightarrow |10\rangle$$
> 2. Apply the regular `CNOT` gate:
> $$|10\rangle \rightarrow |11\rangle$$
> 3. Now, if we don't undo the change we did on the first step, we'll end up with a gate that transforms $|00\rangle$ into $|11\rangle$, which is not the transformation we're trying to implement.
> However, if we undo it by applying the `X` gate to the first qubit again, we'll get the correct state:
> $$|11\rangle \rightarrow |01\rangle$$
>
> You can check that getting the right behavior of the operation on the rest of the basis states also requires that last step.

Finally, let's take a look at a very useful operation [ControlledOnBitString](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonbitstring) provided by the Q# Standard library.
It defines a variant of a gate controlled on a state specified by a bit mask; for example, bit mask `[true, false]` means that the gate should be applied only if the two control qubits are in the $|10\rangle$ state.

The sequence of steps that implement this variant are:
1. Apply the `X` gate to each control qubit that corresponds to a `false` element of the bit mask (in the example, that's just the second qubit). After this, if the control qubits started in the $|10\rangle$ state, they'll end up in the $|11\rangle$ state, and if they started in any other state, they'll end up in any state but $|11\rangle$.
2. Apply the regular controlled version of the gate.
3. Apply the $X$ gate to the same qubits to return them to their original state.

@[exercise]({
    "id": "arbitrary_controls",
    "title": "Arbitrary Controls",
    "descriptionPath": "./arbitrary_controls/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./arbitrary_controls/Verification.qs"
    ],
    "placeholderSourcePath": "./arbitrary_controls/Placeholder.qs",
    "solutionPath": "./arbitrary_controls/solution.md"
})

@[section]({
    "id": "multi_qubit_gates_conclusion",
    "title": "Conclusion"
})

Congratulations! You have completed the series of introductory Katas.
