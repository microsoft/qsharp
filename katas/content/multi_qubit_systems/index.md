# Multi-Qubit Systems

@[section]({
    "id": "multi_qubit_systems_overview",
    "title": "Overview"
})

This kata introduces you to multi-qubit systems - their representation in mathematical notation and in Q# code, and the concept of entanglement.

If you are not familiar with single-qubit systems, we recommend that you complete the qubit kata first.

**This tutorial covers the following topics:**

- Vector representation of multi-qubit systems
- Entangled and separable states
- Dirac notation

**What you should know to start working on this tutorial:**

- Basic single-qubit gates
- The concept of tensor product

@[section]({
    "id": "multi_qubit_systems_introduction",
    "title": "Multi-Qubit Systems"
})

In a previous kata we discussed the concept of a qubit - the basic building block of a quantum computer.
A multi-qubit system is a collection of multiple qubits, treated as a single system.

Let's start by examining a system of two classical bits. Each bit can be in two states: $0$ and $1$. Therefore, a system of two bits can be in four different states: $00$, $01$, $10$, and $11$. Generally, a system of $N$ classical bits can be in any of the $2^N$ states.

A system of $N$ qubits can also be in any of the $2^N$ classical states, but, unlike the classical bits, it can also be in a **superposition** of all these states.

Similarly to single-qubit systems, a state of an $N$-qubit system can be represented as a complex vector of size $2^N$:
$$\begin{bmatrix} x_0 \\\ x_1 \\\ \vdots \\\ x_{2^N-1}\end{bmatrix}$$

## Basis States

Similarly to single-qubit systems, multi-qubit systems have their own sets of basis states.
The computational basis for an $N$-qubit system is a set of $2^N$ vectors, in each of which with one element equals $1$, and the other elements equal $0$.

For example, this is the **computational basis** for a two-qubit system:

<table>
    <tr>
        <td>$$\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix}$$</td>
        <td>$$\begin{bmatrix} 0 \\\ 1 \\\ 0 \\\ 0 \end{bmatrix}$$</td>
        <td>$$\begin{bmatrix} 0 \\\ 0 \\\ 1 \\\ 0 \end{bmatrix}$$</td>
        <td>$$\begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}$$</td>
    </tr>
</table>

It is easy to see that these vectors form an orthonormal basis. Note that each of these basis states can be represented as a tensor product of some combination of single-qubit basis states:

<table>
    <tr>
        <td>$\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix} =
\begin{bmatrix} 1 \\\ 0 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix}$</td>
        <td>$\begin{bmatrix} 0 \\\ 1 \\\ 0 \\\ 0 \end{bmatrix} =
\begin{bmatrix} 1 \\\ 0 \end{bmatrix} \otimes \begin{bmatrix} 0 \\\ 1 \end{bmatrix}$</td>
        <td>$\begin{bmatrix} 0 \\\ 0 \\\ 1 \\\ 0 \end{bmatrix} =
\begin{bmatrix} 0 \\\ 1 \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix}$</td>
        <td>$\begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix} =
\begin{bmatrix} 0 \\\ 1 \end{bmatrix} \otimes \begin{bmatrix} 0 \\\ 1 \end{bmatrix}$</td>
    </tr>
</table>

Any two-qubit system can be expressed as some linear combination of those tensor products of single-qubit basis states.

Similar logic applies to systems of more than two qubits. In general case,

$$
\begin{bmatrix} x_0 \\\ x_1 \\\ \vdots \\\ x_{2^N-1} \end{bmatrix} =
x_0 \begin{bmatrix} 1 \\\ 0 \\\ \vdots \\\ 0 \end{bmatrix} +
x_1 \begin{bmatrix} 0 \\\ 1 \\\ \vdots \\\ 0 \end{bmatrix} + \dotsb +
x_{2^N-1} \begin{bmatrix} 0 \\\ 0 \\\ \vdots \\\ 1 \end{bmatrix}
$$

The coefficients of the basis vectors define how "close" is the system state to the corresponding basis vector.

> Just like with single-qubit systems, there exist other orthonormal bases states for multi-qubit systems. An example for a two-qubit system is the **Bell basis**:
>
> $$\frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}$$
> $$\frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ -1 \end{bmatrix}$$
> $$\frac{1}{\sqrt{2}}\begin{bmatrix} 0 \\\ 1 \\\ 1 \\\ 0 \end{bmatrix}$$
> $$\frac{1}{\sqrt{2}}\begin{bmatrix} 0 \\\ 1 \\\ -1 \\\ 0 \end{bmatrix}$$
>
> You can check that these vectors are normalized, and orthogonal to each other, and that any two-qubit state can be expressed as a linear combination of these vectors.  The vectors of Bell basis, however, can not be represented as tensor products of single-qubit basis states.

@[section]({
    "id": "multi_qubit_systems_separable_states",
    "title": "Separable States"
})

Sometimes the state of a multi-qubit system can be separated into the states of individual qubits or smaller subsystems.
To do this, you would express the vector state of the system as a tensor product of the vectors representing each individual qubit/subsystem.
Here is an example for two qubits:

$$
\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ 0 \\\ \frac{1}{\sqrt{2}} \\\ 0 \end{bmatrix} =
\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ \frac{1}{\sqrt{2}} \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix}
$$

The states that allow such representation are known as **separable states**.

@[question]({
    "id": "separable_state",
    "descriptionPath": "./separable_state/index.md",
    "answerPath": "./separable_state/solution.md"
})

@[question]({
    "id": "is_it_separable",
    "descriptionPath": "./is_it_separable/index.md",
    "answerPath": "./is_it_separable/solution.md"
})

@[section]({
    "id": "multi_qubit_systems_entanglement",
    "title": "Entanglement"
})

As we've just seen, some quantum states are impossible to factor into individual qubit states or even into states of larger subsystems. The states of these qubits are inseparable from one another and must always be considered as part of a larger system - they are **entangled**.

> For example, every state in the Bell basis we saw earlier is an entangled state.

Entanglement is a huge part of what makes quantum computing so powerful.
It allows us to link the qubits so that they stop behaving like individuals and start behaving like a large, more complex system.
In entangled systems, measuring one of the qubits modifies the state of the other qubits, and tells us something about their state.
In the example above, when one of the qubits is measured, we know that the second qubit will end up in the same state.
This property is used extensively in many quantum algorithms.

@[section]({
    "id": "multi_qubit_systems_dirac_notation",
    "title": "Dirac Notation"
})

Just like with single qubits, Dirac notation provides a useful shorthand for writing down states of multi-qubit systems.

As we've seen earlier, multi-qubit systems have their own canonical bases, and the basis states can be represented as tensor products of single-qubit basis states. Any multi-qubit system can be represented as a linear combination of these basis states:

$$
\begin{bmatrix} x_0 \\\ x_1 \\\ x_2 \\\ x_3 \end{bmatrix} =
x_0\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix} +
x_1\begin{bmatrix} 0 \\\ 1 \\\ 0 \\\ 0 \end{bmatrix} +
x_2\begin{bmatrix} 0 \\\ 0 \\\ 1 \\\ 0 \end{bmatrix} +
x_3\begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix} =
x_0|0\rangle \otimes |0\rangle +
x_1|0\rangle \otimes |1\rangle +
x_2|1\rangle \otimes |0\rangle +
x_3|1\rangle \otimes |1\rangle
$$

To simplify this, tensor products of basis states have their own notation:

$$|0\rangle \otimes |0\rangle = |00\rangle$$
$$|0\rangle \otimes |1\rangle = |01\rangle$$
$$|1\rangle \otimes |0\rangle = |10\rangle$$
$$|1\rangle \otimes |1\rangle = |11\rangle$$

$$|0\rangle \otimes |0\rangle \otimes |0\rangle = |000\rangle$$

And so on.

Or, more generally:

$$|i_0\rangle \otimes |i_1\rangle \otimes \dotsb \otimes |i_n\rangle = |i_0i_1...i_n\rangle$$

Using this notation simplifies our example:

$$
\begin{bmatrix} x_0 \\\ x_1 \\\ x_2 \\\ x_3 \end{bmatrix} =
x_0|00\rangle + x_1|01\rangle + x_2|10\rangle + x_3|11\rangle
$$

Just like with single qubits, we can put arbitrary symbols within the kets the same way variables are used in algebra.
Whether a ket represents a single qubit or an entire system depends on the context.
Some ket symbols have a commonly accepted usage, such as the symbols for the Bell basis:

<table>
    <tr>
        <td>$|\phi^+\rangle = \frac{1}{\sqrt{2}}\big(|00\rangle + |11\rangle\big) \\\ |\phi^-\rangle = \frac{1}{\sqrt{2}}\big(|00\rangle - |11\rangle\big)$</td>
        <td>$|\psi^+\rangle = \frac{1}{\sqrt{2}}\big(|01\rangle + |10\rangle\big) \\\ |\psi^-\rangle = \frac{1}{\sqrt{2}}\big(|01\rangle - |10\rangle\big)$</td>
    </tr>
</table>

>## Endianness
>
> In classical computing, endianness refers to the order of bits (or bytes) when representing numbers in binary. You're probably familiar with the typical way of writing numbers in binary: $0 = 0_2$, $1 = 1_2$, $2 = 10_2$, $3 = 11_2$, $4 = 100_2$, $5 = 101_2$, $6 = 110_2$, etc. This is known as **big-endian format**. In big-endian format, the *most significant* bits come first. For example: $110_2 = 1 \cdot 4 + 1 \cdot 2 + 0 \cdot 1 = 4 + 2 = 6$.
>
> There is an alternate way of writing binary numbers - **little-endian format**. In little-endian format, the *least significant* bits come first. For example, $2$ would be written as $01$, $4$ as $001$, and $6$ as $011$. To put it another way, in little endian format, the number is written backwards compared to the big-endian format.
>
> In Dirac notation for multi-qubit systems, it's common to see integer numbers within the kets instead of bit sequences. What those numbers mean depends on the context - whether the notation used is big-endian or little-endian.
>
> Examples with a 3 qubit system:
>
> <table>
>    <tr>
>        <th>Integer Ket</th>
>        <td>$|0\rangle$</td>
>        <td>$|1\rangle$</td>
>        <td>$|2\rangle$</td>
>        <td>$|3\rangle$</td>
>        <td>$|4\rangle$</td>
>        <td>$|5\rangle$</td>
>        <td>$|6\rangle$</td>
>        <td>$|7\rangle$</td>
>    </tr>
>    <tr>
>        <th>Big-endian</th>
>        <td>$|000\rangle$</td>
>        <td>$|001\rangle$</td>
>        <td>$|010\rangle$</td>
>        <td>$|011\rangle$</td>
>        <td>$|100\rangle$</td>
>        <td>$|101\rangle$</td>
>        <td>$|110\rangle$</td>
>        <td>$|111\rangle$</td>
>    </tr>
>    <tr>
>        <th>Little-endian</th>
>        <td>$|000\rangle$</td>
>        <td>$|100\rangle$</td>
>        <td>$|010\rangle$</td>
>        <td>$|110\rangle$</td>
>        <td>$|001\rangle$</td>
>        <td>$|101\rangle$</td>
>        <td>$|011\rangle$</td>
>        <td>$|111\rangle$</td>
>    </tr>
></table>
>
> Multi-qubit quantum systems that store superpositions of numbers are often referred to as **quantum registers**.

@[section]({
    "id": "multi_qubit_systems_in_qsharp",
    "title": "Multi-qubit systems in Q#"
})

This demo shows you how to allocate multiple qubits in Q# and examine their joint state. It uses single-qubit gates for manipulating the individual qubit states - if you need a refresher on them, please review the single-qubit gates kata.

These demos use the function `DumpMachine` to print the state of the quantum simulator.
If you aren't familiar with the output of this function for single qubits, you should revisit the qubit kata.
When printing the state of multi-qubit systems, this function outputs the same information for each multi-qubit basis state.
The qubit kata explains how `DumpMachine` works for multiple qubits in more detail.

@[example]({"id": "multiqubit_system", "codePath": "./examples/MultiQubitSystems.qs"})

> You might have noticed that we've been "resetting" the qubits at the end of our demos, i.e., returning them to $|0\rangle$ state. Q# requires you to return your qubits into the $|0\rangle$ state before releasing them at the end of the `using` block.
> The reason for this is entanglement.
>
> Consider running a program on a quantum computer: the number of qubits is very limited, and you want to reuse the released qubits in other parts of the program.
If they are not in zero state by that time, they can potentially be still entangled with the qubits which are not yet released, thus operations you perform on them can affect the state of other parts of the program, causing erroneous and hard to debug behavior.
>
> Resetting the qubits to zero state automatically when they go outside the scope of their using block is dangerous as well: if they were entangled with others, measuring them to reset them can affect the state of the unreleased qubits, and thus change the results of the program - without the developer noticing this.
>
> The requirement that the qubits should be in zero state before they can be released aims to remind the developer to double-check that all necessary information has been properly extracted from the qubits, and that they are not entangled with unreleased qubits any more.
>
> (An alternative way to break entanglement is to measure qubits; in this case Q# allows to release them regardless of the measurement result. You can learn more about measurements in the qubit kata.)

In the following exercises you will learn to prepare separable quantum states by manipulating individual qubits.
You will only need knowledge from the single-qubit gates kata for that.

> In each exercise, you'll be given an array of qubits to manipulate; you can access $i$-th element of the array `qs` as `qs[i]`.
Array elements are indexed starting with 0, the first array element corresponds to the leftmost qubit in Dirac notation.

@[exercise]({
    "id": "prepare_basis_state",
    "title": "Prepare a basis state",
    "descriptionPath": "./prepare_basis_state/index.md",
    "placeholderSourcePath": "./prepare_basis_state/placeholder.qs",
    "solutionPath": "./prepare_basis_state/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./prepare_basis_state/verification.qs"
    ]
})

@[exercise]({
    "id": "prepare_superposition",
    "title": "Prepare a superposition of two basis states",
    "descriptionPath": "./prepare_superposition/index.md",
    "placeholderSourcePath": "./prepare_superposition/placeholder.qs",
    "solutionPath": "./prepare_superposition/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./prepare_superposition/verification.qs"
    ]
})

@[exercise]({
    "id": "prepare_with_real",
    "title": " Prepare a superposition with real amplitudes",
    "descriptionPath": "./prepare_with_real/index.md",
    "placeholderSourcePath": "./prepare_with_real/placeholder.qs",
    "solutionPath": "./prepare_with_real/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./prepare_with_real/verification.qs"
    ]
})

@[exercise]({
    "id": "prepare_with_complex",
    "title": "Prepare a superposition with complex amplitudes",
    "descriptionPath": "./prepare_with_complex/index.md",
    "placeholderSourcePath": "./prepare_with_complex/placeholder.qs",
    "solutionPath": "./prepare_with_complex/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./prepare_with_complex/verification.qs"
    ]
})

@[section]({
    "id": "multi_qubit_systems_conclusion",
    "title": "Conclusion"
})

As you've seen in the exercises, you can prepare separable multi-qubit states using only single-qubit gates.
However, to prepare and manipulate entangled states you'll need more powerful tools.
In the next kata, multi-qubit gates, you will learn about multi-qubit gates which give you access to all states of multi-qubit systems.
