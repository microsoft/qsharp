# Single-Qubit Gates

This tutorial introduces you to single-qubit gates. Quantum gates are the quantum counterpart to classical logic gates, acting as the building blocks of quantum algorithms. Quantum gates transform qubit states in various ways, and can be applied sequentially to perform complex quantum calculations. Single-qubit gates, as their name implies, act on individual qubits. You can learn more at [Wikipedia](https://en.wikipedia.org/wiki/Quantum_logic_gate).

This tutorial covers the following topics:

- Matrix representation
- Ket-bra representation
- The most important single-qubit gates

## The Basics

There are certain properties common to all quantum gates. This section will introduce those properties, using the $X$ gate as an example.

### Matrix Representation

Quantum gates are represented as $2^N \times 2^N$ [unitary matrices](../LinearAlgebra/LinearAlgebra.ipynb#Unitary-Matrices), where $N$ is the number of qubits the gate operates on. 
As a quick reminder, a unitary matrix is a square matrix whose inverse is its adjoint. 
Single-qubit gates are represented by $2 \times 2$ matrices.
Our example for this section, the $X$ gate, is represented by the following matrix:

$$\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$$

You may recall that the state of a qubit is represented by a vector of size $2$. You can apply a gate to a qubit by [multiplying](../LinearAlgebra/LinearAlgebra.ipynb#Matrix-Multiplication) the gate's matrix by the qubit's state vector. The result will be another vector, representing the new state of the qubit. For example, applying the $X$ gate to the computational basis states looks like this:

$$X|0\rangle =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}
\begin{bmatrix} 1 \\ 0 \end{bmatrix} =
\begin{bmatrix} 0 \cdot 1 + 1 \cdot 0 \\ 1 \cdot 1 + 0 \cdot 0 \end{bmatrix} =
\begin{bmatrix} 0 \\ 1 \end{bmatrix}$$

$$X|1\rangle =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}
\begin{bmatrix} 0 \\ 1 \end{bmatrix} =
\begin{bmatrix} 0 \cdot 0 + 1 \cdot 1 \\ 1 \cdot 0 + 0 \cdot 1 \end{bmatrix} =
\begin{bmatrix} 1 \\ 0 \end{bmatrix}$$

The general case:

$$|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$$

$$X|\psi\rangle =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}
\begin{bmatrix} \alpha \\ \beta \end{bmatrix} =
\begin{bmatrix} 0 \cdot \alpha + 1 \cdot \beta \\ 1 \cdot \alpha + 0 \cdot \beta \end{bmatrix} =
\begin{bmatrix} \beta \\ \alpha \end{bmatrix}$$

> If you need a reminder of what $|0\rangle$, $|1\rangle$, and $|\psi\rangle$ mean, you can review the section on [Dirac notation](../Qubit/Qubit.ipynb#Dirac-Notation) in the previous tutorial.

Because this is the most common way to represent quantum gates, the terms "gate" and "gate matrix" will be used interchangeably in this tutorial.

Applying several quantum gates in sequence is equivalent to performing several of these multiplications. 
For example, if you have gates $A$ and $B$ and a qubit in state $|\psi\rangle$, the result of applying $A$ followed by $B$ to that qubit would be $B\big(A|\psi\rangle\big)$ (the gates closest to the qubit state get applied first). 
Matrix multiplication is associative, so this is equivalent to multiplying the $B$ matrix by the $A$ matrix, producing a compound gate of the two, and then applying that to the qubit: $\big(BA\big)|\psi\rangle$.

All quantum gates are reversible - there is another gate which will undo any given gate's transformation, returning the qubit to its original state. 
This means that when dealing with quantum gates, information about qubit states is never lost, as opposed to classical logic gates, some of which destroy information. 
Quantum gates are represented by unitary matrices, so the inverse of a gate is its adjoint; these terms are also used interchangeably in quantum computing.

### Effects on Basis States (Dirac Notation, Continued)

There is a simple way to find out what a gate does to the two computational basis states ($|0\rangle$ and $|1\rangle$) from looking at its matrix that comes in handy when you want to work with states in Dirac notation. Consider an arbitrary gate:

$$A = \begin{bmatrix} \epsilon & \zeta \\ \eta & \mu \end{bmatrix}$$

Watch what happens when we apply it to these states:

$$A|0\rangle =
\begin{bmatrix} \epsilon & \zeta \\ \eta & \mu \end{bmatrix}
\begin{bmatrix} 1 \\ 0 \end{bmatrix} =
\begin{bmatrix} \epsilon \cdot 1 + \zeta \cdot 0 \\ \eta \cdot 1 + \mu \cdot 0 \end{bmatrix} =
\begin{bmatrix} \epsilon \\ \eta \end{bmatrix} = \epsilon|0\rangle + \eta|1\rangle$$

$$A|1\rangle =
\begin{bmatrix} \epsilon & \zeta \\ \eta & \mu \end{bmatrix}
\begin{bmatrix} 0 \\ 1 \end{bmatrix} =
\begin{bmatrix} \epsilon \cdot 0 + \zeta \cdot 1 \\ \eta \cdot 0 + \mu \cdot 1 \end{bmatrix} =
\begin{bmatrix} \zeta \\ \mu \end{bmatrix} = \zeta|0\rangle + \mu|1\rangle$$

Notice that applying the gate to the $|0\rangle$ state transforms it into the state written as the first column of the gate's matrix. Likewise, applying the gate to the $|1\rangle$ state transforms it into the state written as the second column. This holds true for any quantum gate, including, of course, the $X$ gate:

$$X = \begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$$

$$X|0\rangle = \begin{bmatrix} 0 \\ 1 \end{bmatrix} = |1\rangle$$

$$X|1\rangle = \begin{bmatrix} 1 \\ 0 \end{bmatrix} = |0\rangle$$

Once you understand how a gate affects the computational basis states, you can easily find how it affects any state.
Recall that any qubit state vector can be written as a linear combination of the basis states:

$$|\psi\rangle = \begin{bmatrix} \alpha \\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle$$

Because matrix multiplication distributes over addition, once you know how a gate affects those two basis states, you can calculate how it affects any state:

$$X|\psi\rangle = X\big(\alpha|0\rangle + \beta|1\rangle\big) = X\big(\alpha|0\rangle\big) + X\big(\beta|1\rangle\big) = \alpha X|0\rangle + \beta X|1\rangle = \alpha|1\rangle + \beta|0\rangle$$

That is, applying a gate to a qubit in superposition is equivalent to applying that gate to the basis states that make up that superposition and adding the results with appropriate weights.

## Ket-bra Representation

There is another way to represent quantum gates, this time using Dirac notation. However, the kets we've been using aren't enough to represent arbitrary matrices. We need to introduce another piece of notation: the **bra** (this is why Dirac notation is sometimes called **bra-ket notation**).

Recall that kets represent column vectors; a bra is a ket's row vector counterpart. For any ket $|\psi\rangle$, the corresponding bra is its adjoint (conjugate transpose): $\langle\psi| = |\psi\rangle^\dagger$.

> As a quick reminder, the [adjoint](../LinearAlgebra/LinearAlgebra.ipynb#Unary-Operations), also known as the conjugate transpose of a matrix, well, the conjugate of that matrix's transpose.

Some examples:

| Ket                                                                                  | Bra                                                                                   |
|--------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| $|0\rangle = \begin{bmatrix} 1 \\ 0 \end{bmatrix}$                                   | $\langle0| = \begin{bmatrix} 1 & 0 \end{bmatrix}$                                     |
| $|1\rangle = \begin{bmatrix} 0 \\ 1 \end{bmatrix}$                                   | $\langle1| = \begin{bmatrix} 0 & 1 \end{bmatrix}$                                     |
| $|i\rangle = \begin{bmatrix} \frac{1}{\sqrt{2}} \\ \frac{i}{\sqrt{2}} \end{bmatrix}$ | $\langle i| = \begin{bmatrix} \frac{1}{\sqrt{2}} & -\frac{i}{\sqrt{2}} \end{bmatrix}$ |
| $|\psi\rangle = \begin{bmatrix} \alpha \\ \beta \end{bmatrix}$                       | $\langle\psi| = \begin{bmatrix} \overline{\alpha} & \overline{\beta} \end{bmatrix}$   |
| $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$                                    | $\langle\psi| = \overline{\alpha}\langle0| + \overline{\beta}\langle1|$               |

Kets and bras give us a neat way to express [inner](../LinearAlgebra/LinearAlgebra.ipynb#Inner-Product) and [outer](../LinearAlgebra/LinearAlgebra.ipynb#Outer-Product) products. The inner product of $|\phi\rangle$ and $|\psi\rangle$ is the matrix product of $\langle\phi|$ and $|\psi\rangle$, denoted as $\langle\phi|\psi\rangle$, and their outer product is the matrix product of $|\phi\rangle$ and $\langle\psi|$, denoted as $|\phi\rangle\langle\psi|$. Notice that the norm of $|\psi\rangle$ is $\sqrt{\langle\psi|\psi\rangle}$.

This brings us to representing matrices. Recall that the outer product of two vectors of the same size produces a square matrix. We can use a linear combination of several outer products of simple vectors (such as basis vectors) to express any square matrix. For example, the $X$ gate can be expressed as follows:

$$X = |0\rangle\langle1| + |1\rangle\langle0|$$

$$|0\rangle\langle1| + |1\rangle\langle0| =
\begin{bmatrix} 1 \\ 0 \end{bmatrix}\begin{bmatrix} 0 & 1 \end{bmatrix} +
\begin{bmatrix} 0 \\ 1 \end{bmatrix}\begin{bmatrix} 1 & 0 \end{bmatrix} =
\begin{bmatrix} 0 & 1 \\ 0 & 0 \end{bmatrix} + \begin{bmatrix} 0 & 0 \\ 1 & 0 \end{bmatrix} =
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$$

This representation can be used to carry out calculations in Dirac notation without ever switching back to matrix representation:

$$X|0\rangle = \big(|0\rangle\langle1| + |1\rangle\langle0|\big)|0\rangle = |0\rangle\langle1|0\rangle + |1\rangle\langle0|0\rangle = |0\rangle\big(\langle1|0\rangle\big) + |1\rangle\big(\langle0|0\rangle\big) = |0\rangle(0) + |1\rangle(1) = |1\rangle$$

> That last step may seem a bit confusing. Recall that $|0\rangle$ and $|1\rangle$ form an **orthonormal basis**. That is, they are both normalized, and they are orthogonal to each other.
>
> A vector is normalized if its norm is equal to $1$, which only happens if its inner product with itself is equal to $1$. This means that $\langle0|0\rangle = \langle1|1\rangle = 1$
>
> Two vectors are orthogonal to each other if their inner product equals $0$. This means that $\langle0|1\rangle = \langle 1|0\rangle = 0$.

In general case, a matrix 
$$A = \begin{bmatrix} a_{00} & a_{01} \\ a_{10} & a_{11} \end{bmatrix}$$
will have the following ket-bra representation:
$$A = a_{00} |0\rangle\langle0| + a_{01} |0\rangle\langle1| + a_{10} |1\rangle\langle0| + a_{11} |1\rangle\langle1|$$

> ### Ket-bra decomposition
>
> This section describes a more formal process of finding the ket-bra decompositions of quantum gates. This section is not necessary to start working with quantum gates, so feel free to skip it for now, and come back to it later.
>
> You can use the properties of [eigenvalues and eigenvectors](../LinearAlgebra/LinearAlgebra.ipynb#Part-III:-Eigenvalues-and-Eigenvectors) to find the ket-bra decomposition of any gate. Given a gate $A$, and its orthogonal eigenvectors $|\phi\rangle$ and $|\psi\rangle$, if:
>
> $$A|\phi\rangle = x_\phi|\phi\rangle$$
> $$A|\psi\rangle = x_\psi|\psi\rangle$$
>
> Then:
>
> $$A = x_\phi|\phi\rangle\langle\phi| + x_\psi|\psi\rangle\langle\psi|$$
>
> Let's use our $X$ gate as a simple example. The $X$ gate has two eigenvectors: $|+\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + |1\rangle\big)$ and $|-\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big)$. Their eigenvalues are $1$ and $-1$ respectively:
>
> $$X|+\rangle = |+\rangle$$
> $$X|-\rangle = -|-\rangle$$
>
> Here's what the decomposition looks like:
>
> $$X = |+\rangle\langle+| - |-\rangle\langle-| =$$
> $$= \frac{1}{2}\big[\big(|0\rangle + |1\rangle\big)\big(\langle0| + \langle1|\big) - \big(|0\rangle - |1\rangle\big)\big(\langle0| - \langle1|\big)\big] =$$
> $$= \frac{1}{2}\big({\color{red}{|0\rangle\langle0|}} + |0\rangle\langle1| + |1\rangle\langle0| + {\color{red}{|1\rangle\langle1|}} - {\color{red}{|0\rangle\langle0|}} + |0\rangle\langle1| + |1\rangle\langle0| - {\color{red}{|1\rangle\langle1|}}\big) =$$
> $$= \frac{1}{2}\big(2|0\rangle\langle1| + 2|1\rangle\langle0|\big) = |0\rangle\langle1| + |1\rangle\langle0|$$

### Important Gates

This section introduces some of the common single-qubit gates, including their matrix form, their ket-bra decomposition, and a brief "cheatsheet" listing their effect on some common qubit states.

You can use a tool called [Quirk](https://algassert.com/quirk) to visualize how these gates interact with various qubit states.

This section relies on the following notation:

| $|+\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + |1\rangle\big)$  | $|-\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big)$   |
|------------------------------------------------------------------|-------------------------------------------------------------------|
| $|i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + i|1\rangle\big)$ | $|-i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - i|1\rangle\big)$ |

### Pauli Gates

The Pauli gates, named after [Wolfgang Pauli](https://en.wikipedia.org/wiki/Wolfgang_Pauli), are based on the so-called **Pauli matrices**. All three Pauli gates are **self-adjoint**, meaning that each one is its own inverse.

| Gate | Matrix                                          | Ket-Bra                                      | Applying to $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$ | Applying to basis states                                                                                                                                            | Q# Documentation |
|------|-------------------------------------------------|----------------------------------------------|---------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------|
| $X$  | $\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$  | $|0\rangle\langle1| + |1\rangle\langle0|$    | $X|\psi\rangle = \alpha|1\rangle + \beta|0\rangle$            | $X|0\rangle = |1\rangle \\
X|1\rangle = |0\rangle \\
X|+\rangle = |+\rangle \\
X|-\rangle = -|-\rangle \\
X|i\rangle = i|-i\rangle \\
X|-i\rangle = -i|i\rangle$    | X                |
| $Y$  | $\begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix}$ | $i(|1\rangle\langle0| - |0\rangle\langle1|)$ | $Y|\psi\rangle = i\big(\alpha|1\rangle - \beta|0\rangle\big)$ | $Y|0\rangle = i|1\rangle \\
Y|1\rangle = -i|0\rangle \\
Y|+\rangle = -i|-\rangle \\
Y|-\rangle = i|+\rangle \\
Y|i\rangle = |i\rangle \\
Y|-i\rangle = -|-i\rangle$ | Y                |
| $Z$  | $\begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$ | $|0\rangle\langle0| - |1\rangle\langle1|$    | $Z|\psi\rangle = \alpha|0\rangle - \beta|1\rangle$            | $Z|0\rangle = |0\rangle \\
Z|1\rangle = -|1\rangle \\
Z|+\rangle = |-\rangle \\
Z|-\rangle = |+\rangle \\
Z|i\rangle = |-i\rangle \\
Z|-i\rangle = |i\rangle$       | Z                |

> The $X$ gate is sometimes referred to as the **bit flip** gate, or the **NOT** gate, because it acts like the classical NOT gate on the computational basis.
>
> The $Z$ gate is sometimes referred to as the **phase flip** gate.

Here are several properties of the Pauli gates that are easy to verify and convenient to remember:

- Different Pauli gates *anti-commute*:
  $$XZ = -ZX, XY = -YX, YZ = -ZY$$
- A product of any two Pauli gates equals the third gate, with an extra $i$ (or $-i$) phase:
  $$XY = iZ, YZ = iX, ZX = iY$$ 
- A product of all three Pauli gates equals identity (with an extra $i$ phase):
  $$XYZ = iI$$

#### Demo: Pauli Gates

The following cell contains code demonstrating how to apply gates in Q#, using the Pauli $X$ gate as an example. It sets up a series of quantum states, and then shows the result of applying the $X$ gate to each one. To run the demo, run the next cell using `Ctrl+Enter` (`⌘+Enter` on a Mac).

In the previous tutorial we discussed that the qubit state in Q# cannot be directly assigned or accessed. The same logic is extended to the quantum gates: applying a gate to a qubit modifies the internal state of that qubit but doesn't return the resulting state of the qubit. This is why we never assign the output of these gates to any variables in this demo - they don't produce any output.

Applying several gates in a row follows the same principle. In the mathematical notation applying an $X$ gate followed by a $Z$ gate to a state $|\psi\rangle$ is denoted as $Z(X(|\psi\rangle))$, because the result of applying a gate to a state is another state. In Q#, applying a gate doesn't return anything, so you can't use its output as an input to another gate - something like `Z(X(q))` will not produce expected result. Instead, to apply several gates to the same qubit, you need to call them separately in the order in which they are applied:

```qsharp
X(q);
Z(q);
```

All the basic gates we will be covering in this tutorial are part of the [Intrinsic](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic) namespace. We're also using the function [DumpMachine](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.diagnostics.dumpmachine) to print the state of the quantum simulator.
```qsharp
// Run this cell using Ctrl+Enter (⌘+Enter on Mac)
// Run the next cell to see the output

// To use a namespace, you need to use the `open` keyword to access it
open Microsoft.Quantum.Diagnostics;

operation PauliGatesDemo () : Unit {
    // This allocates a qubit for us to work with
    use q = Qubit();

    // This will put the qubit into an uneven superposition |𝜓❭,
    // where the amplitudes of |0⟩ and |1⟩ have different moduli
    Ry(1.0, q);

    Message("Qubit in state |𝜓❭:");
    DumpMachine();

    // Let's apply the X gate; notice how it swaps the amplitudes of the |0❭ and |1❭ basis states
    X(q);
    Message("Qubit in state X|𝜓❭:");
    DumpMachine();

    // Applying the Z gate adds -1 relative phase to the |1❭ basis states
    Z(q);
    Message("Qubit in state ZX|𝜓❭:");
    DumpMachine();

    // Finally, applying the Y gate returns the qubit to its original state |𝜓❭, with an extra global phase of i
    Y(q);
    Message("Qubit in state YZX|𝜓❭:");
    DumpMachine();

    // This returns the qubit into state |0❭
    Reset(q);
}
```

In the previous tutorials we used `%simulate` command to run the Q# code on the full-state simulator. Here we will use an additional `%trace` command: it will print the circuit diagram of the run after the output.

```qsharp
%simulate PauliGatesDemo
%trace PauliGatesDemo
```

## Exercises

### The $Y$ gate

**Input:** A qubit in an arbitrary state $|\\psi\\rangle = \\alpha|0\\rangle + \\beta|1\\rangle$.

**Goal:** Apply the Y gate to the qubit, i.e., transform the given state into $i\\alpha|1\\rangle - i\\beta|0\\rangle$.

@[exercise]({
    "id": "y_gate",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./y_gate/Verification.qs",
    "placeholderSourcePath": "./y_gate/Placeholder.qs",
    "solutionSourcePath": "./y_gate/Solution.qs",
    "solutionDescriptionPath": "./y_gate/solution.md"
})

### Applying a global phase $i$

**Input:** A qubit in an arbitrary state $|\\psi\\rangle = \\alpha|0\\rangle + \\beta|1\\rangle$.

**Goal:** Use several Pauli gates to change the qubit state to $i|\\psi\\rangle = i\\alpha|0\\rangle + i\\beta|1\\rangle$.

@[exercise]({
    "id": "global_phase_i",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./global_phase_i/Verification.qs",
    "placeholderSourcePath": "./global_phase_i/Placeholder.qs",
    "solutionSourcePath": "./global_phase_i/Solution.qs",
    "solutionDescriptionPath": "./global_phase_i/solution.md"
})

### Applying a $-1$ phase to $|0\rangle$ state

**Input:** A qubit in an arbitrary state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$.

**Goal:** Use several Pauli gates to change the qubit state to $- \alpha|0\rangle + \beta|1\rangle$, i.e., apply the transformation represented by the following matrix::

$$\begin{bmatrix} -1 & 0 \\ 0 & 1 \end{bmatrix}$$

@[exercise]({
    "id": "sign_flip_on_zero",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./sign_flip_on_zero/Verification.qs",
    "placeholderSourcePath": "./sign_flip_on_zero/Placeholder.qs",
    "solutionSourcePath": "./sign_flip_on_zero/Solution.qs",
    "solutionDescriptionPath": "./sign_flip_on_zero/solution.md"
})

### Preparing a $|-\rangle$ state

**Input:** A qubit in state $|0\rangle$.

**Goal:** Transform the qubit into state $|-\rangle$.

@[exercise]({
    "id": "prepare_minus",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./prepare_minus/Verification.qs",
    "placeholderSourcePath": "./prepare_minus/Placeholder.qs",
    "solutionSourcePath": "./prepare_minus/Solution.qs",
    "solutionDescriptionPath": "./prepare_minus/solution.md"
})

### Three-fourths phase

**Input:** A qubit in an arbitrary state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$.

**Goal:** Use several phase shift gates to apply the transformation represented by the following matrix to the given qubit:

$$\begin{bmatrix} 1 & 0 \\ 0 & e^{3i\pi/4} \end{bmatrix}$$

@[exercise]({
    "id": "three_quarters_pi_phase",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./three_quarters_pi_phase/Verification.qs",
    "placeholderSourcePath": "./three_quarters_pi_phase/Placeholder.qs",
    "solutionSourcePath": "./three_quarters_pi_phase/Solution.qs",
    "solutionDescriptionPath": "./three_quarters_pi_phase/solution.md"
})

### Preparing a rotated state

**Inputs:**

1. Real numbers $\alpha$ and $\beta$ such that $\alpha^2 + \beta^2 = 1$.
2. A qubit in state $|0\rangle$.

**Goal:** Use a rotation gate to transform the qubit into state $\alpha|0\rangle -i\beta|1\rangle$.

> You will probably need functions from the [Math](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math) namespace, specifically [ArcTan2](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arctan2).
>
> You can assign variables in Q# by using the `let` keyword: `let num = 3;` or `let result = Function(input);`

@[exercise]({
    "id": "prepare_rotated_state",
    "codeDependenciesPaths": [
        "../KatasLibrary.qs"
    ],
    "verificationSourcePath": "./prepare_rotated_state/Verification.qs",
    "placeholderSourcePath": "./prepare_rotated_state/Placeholder.qs",
    "solutionSourcePath": "./prepare_rotated_state/Solution.qs",
    "solutionDescriptionPath": "./prepare_rotated_state/solution.md"
})
