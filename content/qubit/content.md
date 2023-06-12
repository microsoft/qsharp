# The Qubit

This tutorial introduces you to one of the core concepts in quantum computing - the qubit, and its representation in mathematical notation and in Q# code.

This tutorial assumes familiarity with complex arithmetic and linear algebra.

This tutorial covers the following topics:

- The concept of a qubit
- Superposition
- Vector representation of qubit states
- Dirac notation
- `Qubit` data type in Q#"

## The Concept of a Qubit

The basic building block of a classical computer is the bit - a single memory cell that is either in state $0$ or in state $1$. Similarly, the basic building block of a quantum computer is the quantum bit, or **qubit**. Like the classical bit, a qubit can be in state $0$ or in state $1$. Unlike the classical bit, however, the qubit isn't limited to just those two states - it may also be in a combination, or **superposition** of those states.

> A common misconception about quantum computing is that a qubit is always in one state or the other, we just don't know which one until we "measure" it. That is not the case. A qubit in a superposition is in a state between the states $0$ and $1$. When a qubit is measured, it is forced entirely into one state or the other - in other words, measuring it actually changes its state.

## Matrix Representation

The state of a qubit is represented by a complex vector of size 2:

$$\begin{bmatrix} \alpha \\\ \beta \end{bmatrix}$$

Here $\alpha$ represents how "close" the qubit is to the state $0$, and $\beta$ represents how "close" the qubit is to the state $1$. This vector is normalized: $|\alpha|^2 + |\beta|^2 = 1$.

$\alpha$ and $\beta$ are known as **amplitudes** of states $0$ and $1$, respectively.

## Basis States

A qubit in state $0$ would be represented by the following vector:

$$\begin{bmatrix} 1 \\\ 0 \end{bmatrix}$$

Likewise, a qubit in state $1$ would be represented by this vector:

$$\begin{bmatrix} 0 \\\ 1 \end{bmatrix}$$

Note that you can use scalar multiplication and vector addition to express any qubit state as a sum of these two vectors with certain weights (known as **linear combination**):

$$
\begin{bmatrix} \alpha \\\ \beta \\end{bmatrix} =
\begin{bmatrix} \alpha \\\ 0 \end{bmatrix} + \begin{bmatrix} 0 \\\ \beta \\end{bmatrix} =
\alpha \cdot \begin{bmatrix} 1 \\\ 0 \end{bmatrix} + \beta \cdot \begin{bmatrix} 0 \\\ 1 \end{bmatrix}
$$

Because of this, these two states are known as **basis states**.

These two vectors have two additional properties. First, as mentioned before, both are **normalized**:

$$
\langle \begin{bmatrix} 1 \\\ 0 \end{bmatrix} , \begin{bmatrix} 1 \\\ 0 \end{bmatrix} \rangle =
\langle \begin{bmatrix} 0 \\\ 1 \end{bmatrix} , \begin{bmatrix} 0 \\\ 1 \end{bmatrix} \rangle = 1
$$

Second, they are **orthogonal** to each other:

$$
\langle \begin{bmatrix} 1 \\\ 0 \end{bmatrix} , \begin{bmatrix} 0 \\\ 1 \end{bmatrix} \rangle =
\langle \begin{bmatrix} 0 \\\ 1 \end{bmatrix} , \begin{bmatrix} 1 \\\ 0 \end{bmatrix} \\rangle = 0
$$

> As a reminder, $\langle V , W \rangle$ is the inner product of $V$ and $W$.

This means that these vectors form an **orthonormal basis**. The basis of $\begin{bmatrix} 1 \\\ 0 \end{bmatrix}$ and $\begin{bmatrix} 0 \\\ 1 \end{bmatrix}$ is called the **computational basis**, also known as the **canonical basis**.

> There exist other orthonormal bases, for example, the **Hadamard basis**, formed by the vectors
>
> $$\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ \frac{1}{\sqrt{2}} \end{bmatrix} \text{ and } \begin{bmatrix} \frac{1}{\sqrt{2}} \\\ -\frac{1}{\sqrt{2}} \end{bmatrix}$$
>
> You can check that these vectors are normalized, and orthogonal to each other. Any qubit state can be expressed as a linear combination of these vectors:
>
> $$
> \begin{bmatrix} \alpha \\\ \beta \end{bmatrix} =
> \frac{\alpha + \beta}{\sqrt{2}} \begin{bmatrix} \frac{1}{\sqrt{2}} \\\ \frac{1}{\sqrt{2}} \end{bmatrix} +
> \frac{\alpha - \beta}{\sqrt{2}} \begin{bmatrix} \frac{1}{\sqrt{2}} \\\ -\frac{1}{\sqrt{2}} \end{bmatrix}
> $$
>
> The Hadamard basis is widely used in quantum computing, for example, in the [BB84 quantum key distribution protocol](https://en.wikipedia.org/wiki/BB84).

## Dirac Notation

Writing out each vector when doing quantum calculations takes up a lot of space, and this will get even worse once we introduce quantum gates and multi-qubit systems. **Dirac notation** is a shorthand notation that helps solve this issue. In Dirac notation, a vector is denoted by a symbol called a **ket**. For example, a qubit in state $0$ is represented by the ket $|0\rangle$, and a qubit in state $1$ is represented by the ket $|1\rangle$:

<table>
    <col width=150>
    <col width=150>
    <tr>
        <td style=\"text-align:center; border:1px solid\">$|0\rangle = \begin{bmatrix} 1 \\\ 0 \end{bmatrix}$</td>
        <td style=\"text-align:center; border:1px solid\">$|1\rangle = \begin{bmatrix} 0 \\\ 1 \end{bmatrix}$</td>
    </tr>
</table>

These two kets represent basis states, so they can be used to represent any other state:

$$\begin{bmatrix} \alpha \\\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle$$

Any symbol other than $0$ or $1$ within the ket can be used to represent arbitrary vectors, similar to how variables are used in algebra:

$$|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$$

Several ket symbols have a generally accepted use, such as:

<table>
    <col width=180>
    <col width=180>
    <tr>
        <td style=\"text-align:center; border:1px solid\">$|+\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + |1\rangle\big)$</td>
        <td style=\"text-align:center; border:1px solid\">$|-\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big)$</td>
    </tr>
    <tr>
        <td style=\"text-align:center; border:1px solid\">$|i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + i|1\rangle\big)$</td>
        <td style=\"text-align:center; border:1px solid\">$|-i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - i|1\rangle\big)$</td>
    </tr>
</table>

We will learn more about Dirac notation in the next tutorials, as we introduce quantum gates and multi-qubit systems.
