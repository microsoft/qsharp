# Multi-Qubit Systems

@[section]({
    "id": "multi_qubit_systems__overview",
    "title": "Overview"
})

This kata introduces you to multi-qubit systems, their representation in mathematical notation and in Q# code, and the concept of entanglement.

If you are not familiar with single-qubit systems, we recommend that you complete "The Qubit" kata first.

**This kata covers the following topics:**

- Vector representation of multi-qubit systems
- Entangled and separable states
- Dirac notation for multi-qubit systems

**What you should know to start working on this kata:**

- Basic single-qubit gates
- The concept of tensor product

@[section]({
    "id": "multi_qubit_systems__introduction",
    "title": "Multi-Qubit Systems"
})

In The Qubit kata we discussed the concept of a qubit - the basic building block of a quantum computer.
A multi-qubit system is a collection of multiple qubits, treated as a single system.

Let's start by examining a system of two classical bits. Each bit can be in two states: $0$ and $1$. Therefore, a system of two bits can be in four different states: $00$, $01$, $10$, and $11$. Generally, a system of $N$ classical bits can be in any of the $2^N$ states.

A system of $N$ qubits can also be in any of the $2^N$ classical states, but, unlike the classical bits, it can also be in a **superposition** of all these states.

Similarly to single-qubit systems, a state of an $N$-qubit system can be represented as a complex vector of size $2^N$:
$$\begin{bmatrix} x_0 \\\ x_1 \\\ \vdots \\\ x_{2^N-1}\end{bmatrix}$$

## Basis States

Similarly to single-qubit systems, multi-qubit systems have their own sets of basis states.
The computational basis for an $N$-qubit system is a set of $2^N$ vectors, in each of which with one element equals $1$, and the other elements equal $0$.

For example, this is the **computational basis** for a two-qubit system:

$$\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix},
\begin{bmatrix} 0 \\\ 1 \\\ 0 \\\ 0 \end{bmatrix},
\begin{bmatrix} 0 \\\ 0 \\\ 1 \\\ 0 \end{bmatrix},
\begin{bmatrix} 0 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}$$

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
> $$\frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}, 
\frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ -1 \end{bmatrix},
\frac{1}{\sqrt{2}}\begin{bmatrix} 0 \\\ 1 \\\ 1 \\\ 0 \end{bmatrix},
\frac{1}{\sqrt{2}}\begin{bmatrix} 0 \\\ 1 \\\ -1 \\\ 0 \end{bmatrix}$$
>
> You can check that these vectors are normalized, and orthogonal to each other, and that any two-qubit state can be expressed as a linear combination of these vectors.  The vectors of Bell basis, however, can not be represented as tensor products of single-qubit basis states.

@[section]({
    "id": "multi_qubit_systems__separable_states",
    "title": "Separable States"
})

Sometimes the global state of a multi-qubit system can be separated into the states of individual qubits or subsystems. To do this, you would express the vector state of the global system as a tensor product of the vectors representing each individual qubit/subsystem. Here is an example of a two-qubit state:

$$
\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ 0 \\\ \frac{1}{\sqrt{2}} \\\ 0 \end{bmatrix} =
\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ \frac{1}{\sqrt{2}} \end{bmatrix} \otimes \begin{bmatrix} 1 \\\ 0 \end{bmatrix}
$$

You can see that the first qubit is in state $\frac{1}{\sqrt{2}}\big(|0\rangle + |1\rangle\big)$ and the second qubit is in state $|0\rangle$. The multi-qubit states that allow such representation are known as **separable states**, or product states, because you can separate the global state into the tensor product of individual subsystems.


## 🔎 Analyze

Show that the state is separable:
$$
\frac{1}{2} \begin{bmatrix} 1 \\\ i \\\ -i \\\ 1 \end{bmatrix} =
\begin{bmatrix} ? \\\ ? \end{bmatrix} \otimes \begin{bmatrix} ? \\\ ? \end{bmatrix}
$$

<details>
<summary><b>Solution</b></summary>
To separate the state into a tensor product of two single-qubit states, we need to represent it in the following way:

$$
\begin{bmatrix} \alpha \gamma \\\ \alpha \delta \\\ \beta \gamma \\\ \beta \delta \end{bmatrix} = 
\begin{bmatrix} \alpha \\\ \beta \end{bmatrix} \otimes \begin{bmatrix} \gamma \\\ \delta \end{bmatrix}
$$

This brings us to a system of equations:

$$
\begin{cases}
\alpha\gamma = \frac{1}{2} \\\ \alpha\delta = \frac{i}{2} \\\ \beta \gamma = \frac{-i}{2} \\\ \beta \delta = \frac{1}{2}
\end{cases}
$$

Solving this system of equations gives us the answer:

$$\alpha = \frac{1}{\sqrt2}, \beta = \frac{-i}{\sqrt2}, \gamma = \frac{1}{\sqrt2}, \delta = \frac{i}{\sqrt2}$$

$$
\frac{1}{2} \begin{bmatrix} 1 \\\ i \\\ -i \\\ 1 \end{bmatrix} =
\frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ -i \end{bmatrix} \otimes \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ i \end{bmatrix}
$$

Note that finding such representation is not always possible, as you will see in the next exercise.
</details>

## 🔎 Analyze

Is this state separable?

$$\frac{1}{\sqrt{2}}\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 1 \end{bmatrix}$$

<details>
<summary><b>Solution</b></summary>
Let's assume that this state is separable and write down the system of equations to determine the coefficients of individual qubit states in the tensor product, similar to what we did in the previous exercise:

$$
\begin{cases}
\alpha\gamma = \frac{1}{\sqrt2} \\\ \alpha\delta = 0 \\\ \beta \gamma = 0 \\\ \beta \delta = \frac{1}{\sqrt2}
\end{cases}
$$

Now let's multiply the first and the last equations, and the second and the third equations:

$$
\begin{cases}
\alpha\beta\gamma\delta = \frac{1}{2} \\\ \alpha\beta\gamma\delta = 0
\end{cases}
$$

We can see that this system of equations doesn't have a solution, which means that this state is <b>not separable</b>.
</details>

@[section]({
    "id": "multi_qubit_systems__entanglement",
    "title": "Entanglement"
})

Sometimes, quantum states cannot be written as individual qubit states. Quantum systems that are not separable are called **entangled** systems. If a state can be written as the product state of the individual subsystems, that state is not entangled.

Entanglement is a quantum correlation, which is very different from classical correlations. In entanglement, the state of the subsystems isn't determined, and you can talk only about the probabilities associated with the outcomes. The global system must be considered as one.

> For example, every state in the Bell basis is an entangled state.

Entanglement is a huge part of what makes quantum computing so powerful. It allows us to link the qubits so that they stop behaving like individuals and start behaving like a large, more complex system. In entangled systems, measuring one of the qubits modifies the state of the other qubits, and tells us something about their state.

For example, consider two qubits $A$ and $B$ in superposition such that the state of the global system is

$$|\psi\rangle_{AB} = \frac{1}{\sqrt2}|00\rangle + \frac{1}{\sqrt2}|11\rangle$$

In such a state, only two outcomes are possible when you measure the state of both qubits in the standard basis: $|00\rangle$ and $|11\rangle$. Notice that each outcome has the same probability of $\frac{1}{2}$. There's zero probability of obtaining $|01\rangle$ and $|10\rangle$. If you measure the first qubit and you get that it is in $|0\rangle$ state, then you can be positive that the second qubit is also in $|0\rangle$ state, even without measuring it. The measurement outcomes are correlated, and the qubits are entangled.

This property is used extensively in many quantum algorithms.

@[section]({
    "id": "multi_qubit_systems__dirac_notation",
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

$$|\phi^+\rangle = \frac{1}{\sqrt{2}}\big(|00\rangle + |11\rangle\big)$$
$$|\phi^-\rangle = \frac{1}{\sqrt{2}}\big(|00\rangle - |11\rangle\big)$$
$$|\psi^+\rangle = \frac{1}{\sqrt{2}}\big(|01\rangle + |10\rangle\big)$$
$$|\psi^-\rangle = \frac{1}{\sqrt{2}}\big(|01\rangle - |10\rangle\big)$$


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
    "id": "multi_qubit_systems__in_qsharp",
    "title": "Multi-Qubit Systems in Q#"
})

This demo shows you how to allocate multiple qubits in Q# and examine their joint state. It uses single-qubit gates for manipulating the individual qubit states - if you need a refresher on them, please review the Single-Qubit Gates kata.

These demos use the function `DumpMachine` to print the state of the quantum simulator.
When dealing with multi-qubit systems, `DumpMachine` prints information about each basis state that has a non-zero amplitude, one basis state per row, the same as it does for single-qubit systems. 
The basis states are represented as bit strings, one bit per the qubit allocated, with the leftmost bit corresponding 
to the qubit that was allocated the earliest. (If the qubits were allocated at once as an array, the leftmost bit corresponds 
to the first element of the array.)

@[example]({"id": "multi_qubit_systems__multi_qubit_systems_demo", "codePath": "./examples/MultiQubitSystems.qs"})

> You might have noticed that we've been "resetting" the qubits at the end of our demos, that is, returning them to $|0\rangle$ state. Q# requires you to return your qubits into the $|0\rangle$ state before they are released at the end of their scope.
> The reason for this is entanglement.
>
> Consider running a program on a quantum computer: the number of qubits is very limited, and you want to reuse the released qubits in other parts of the program.
If they are not in zero state by that time, they can potentially be still entangled with the qubits which are not yet released, thus operations you perform on them can affect the state of other parts of the program, causing erroneous and hard to debug behavior.
>
> Resetting the qubits to zero state automatically when they go outside the scope of the block they were allocated in is dangerous as well: if they were entangled with others, measuring them to reset them can affect the state of the unreleased qubits, and thus change the results of the program - without the developer noticing this.
>
> The requirement that the qubits should be in zero state before they can be released aims to remind the developer to double-check that all necessary information has been properly extracted from the qubits, and that they are not entangled with unreleased qubits any more.

@[exercise]({
    "id": "multi_qubit_systems__learn_basis_state_amplitudes",
    "title": "Learn Basis State Amplitudes Using DumpMachine",
    "descriptionPath": "./learn_basis_state_amplitudes/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./learn_basis_state_amplitudes/verification.qs"
    ],
    "placeholderSourcePath": "./learn_basis_state_amplitudes/placeholder.qs",
    "solutionPath": "./learn_basis_state_amplitudes/solution.md"
})

@[section]({
    "id": "multi_qubit_systems__exercises",
    "title": "Separable State Preparation"
})

In the following exercises you will learn to prepare separable quantum states by manipulating individual qubits.
You will only need knowledge from the Single-Qubit Gates kata for that.

> In each exercise, you'll be given an array of qubits to manipulate; you can access $i$-th element of the array `qs` as `qs[i]`.
Array elements are indexed starting with 0, the first array element corresponds to the leftmost qubit in Dirac notation.

@[exercise]({
    "id": "multi_qubit_systems__prepare_basis_state",
    "title": "Prepare a Basis State",
    "descriptionPath": "./prepare_basis_state/index.md",
    "placeholderSourcePath": "./prepare_basis_state/placeholder.qs",
    "solutionPath": "./prepare_basis_state/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./prepare_basis_state/verification.qs"
    ]
})

@[exercise]({
    "id": "multi_qubit_systems__prepare_superposition",
    "title": "Prepare a Superposition of Two Basis States",
    "descriptionPath": "./prepare_superposition/index.md",
    "placeholderSourcePath": "./prepare_superposition/placeholder.qs",
    "solutionPath": "./prepare_superposition/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./prepare_superposition/verification.qs"
    ]
})

@[exercise]({
    "id": "multi_qubit_systems__prepare_with_real",
    "title": " Prepare a Superposition with Real Amplitudes",
    "descriptionPath": "./prepare_with_real/index.md",
    "placeholderSourcePath": "./prepare_with_real/placeholder.qs",
    "solutionPath": "./prepare_with_real/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./prepare_with_real/verification.qs"
    ]
})

@[exercise]({
    "id": "multi_qubit_systems__prepare_with_complex",
    "title": "Prepare a Superposition with Complex Amplitudes",
    "descriptionPath": "./prepare_with_complex/index.md",
    "placeholderSourcePath": "./prepare_with_complex/placeholder.qs",
    "solutionPath": "./prepare_with_complex/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./prepare_with_complex/verification.qs"
    ]
})

@[section]({
    "id": "multi_qubit_systems__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to prepare separable multi-qubit states using only single-qubit gates. You also learned the difference between separable states and entangled states. Here are a few key concepts to keep in mind:

- A system of $N$ qubits can be in a superposition of $2^N$ quantum states. The computational basis for an $N$-qubit system is a set of $2^N$ vectors.
- Any two-qubit system can be expressed as some linear combination of the tensor products of single-qubit basis states.
- Two qubits are entangled if their states are correlated and if they can't be described as two independent qubits.

Next, you will learn about multi-qubit gates and how they can give you access to all states of multi-qubit systems in the "Multi-Qubit Gates" kata.
