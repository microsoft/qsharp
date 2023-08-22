# The Qubit

@[section]({
    "id": "qubit_overview",
    "title": "Overview"
})

This Kata introduces you to one of the core concepts in quantum computing - the qubit, and its representation in mathematical notation and in Q# code.

**This Kata covers the following topics:**

- The concept of a qubit
- Superposition
- Vector representation of qubit states
- Dirac notation
- Relative and Global Phase
- `Qubit` data type in Q#
- Visualizing Quantum State using `DumpMachine`

**What you should know to start working on this Kata:**

- Complex arithmetic
- Linear algebra

@[section]({
    "id": "qubit_concept",
    "title": "The Concept of Qubit"
})

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

@[section]({
    "id": "qubit_dirac_notation",
    "title": "Dirac Notation"
})

Writing out each vector when doing quantum calculations takes up a lot of space, and this will get even worse once we introduce quantum gates and multi-qubit systems. **Dirac notation** is a shorthand notation that helps solve this issue. In Dirac notation, a vector is denoted by a symbol called a **ket**. For example, a qubit in state $0$ is represented by the ket $|0\rangle$, and a qubit in state $1$ is represented by the ket $|1\rangle$:

<table>
    <tr>
        <td>$|0\rangle = \begin{bmatrix} 1 \\\ 0 \end{bmatrix}$</td>
        <td>$|1\rangle = \begin{bmatrix} 0 \\\ 1 \end{bmatrix}$</td>
    </tr>
</table>

These two kets represent basis states, so they can be used to represent any other state:

$$\begin{bmatrix} \alpha \\\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle$$

Any symbol other than $0$ or $1$ within the ket can be used to represent arbitrary vectors, similar to how variables are used in algebra:

$$|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$$

Several ket symbols have a generally accepted use, such as:

<table>
    <tr>
        <td>$|+\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + |1\rangle\big)$</td>
        <td>$|-\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big)$</td>
    </tr>
    <tr>
        <td>$|i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle + i|1\rangle\big)$</td>
        <td>$|-i\rangle = \frac{1}{\sqrt{2}}\big(|0\rangle - i|1\rangle\big)$</td>
    </tr>
</table>

We will learn more about Dirac notation in the next Katas, as we introduce quantum gates and multi-qubit systems.


@[section]({
    "id": "qubit_relative_and_global_phase",
    "title": "Relative and Global Phase"
})

You may recall that a complex number has a parameter called its phase. If a complex number $x$ is written in polar form $x = re^{i\theta}$, its phase is $\theta$.

The phase of a basis state is the complex phase of the amplitude of that state. For example, a system in state $\frac{1 + i}{2}|0\rangle + \frac{1 - i}{2}|1\rangle$, the phase of $|0\rangle$ is $\frac{\pi}{4}$, and the phase of $|1\rangle$ is $-\frac{\pi}{4}$. The difference between these two phases is known as **relative phase**.

Multiplying the state of the entire system by $e^{i\theta}$ doesn't affect the relative phase: $\alpha|0\rangle + \beta|1\rangle$ has the same relative phase as $e^{i\theta}\big(\alpha|0\rangle + \beta|1\rangle\big)$. In the second expression, $\theta$ is known as the system's **global phase**.

The state of a qubit (or, more generally, the state of a quantum system) is defined by its relative phase - global phase arises as a consequence of using linear algebra to represent qubits, and has no physical meaning. That is, applying a phase to the entire state of a system (multiplying the entire vector by $e^{i\theta}$ for any real $\theta$) doesn't actually affect the state of the system. Because of this, global phase is sometimes known as **unobservable phase** or **hidden phase**.

@[section]({
    "id": "qubit_qsharp_data_type",
    "title": "Q# Qubit Data Type"
})

In Q#, qubits are represented by the `Qubit` data type. On a physical quantum computer, it's impossible to directly access the state of a qubit, whether to read its exact state, or to set it to a desired state, and this data type reflects that. Instead, you can change the state of a qubit using quantum gates, and extract information about the state of the system using measurements.

That being said, when you run Q# code on a quantum simulator instead of a physical quantum computer, you can use diagnostic functions that allow you to peek at the state of the quantum system. This is very useful both for learning and for debugging small Q# programs.

The qubits aren't an ordinary data type, so the variables of this type have to be declared and initialized ("allocated") a little differently.

Freshly allocated qubits start out in state $|0\rangle$, and have to be returned to that state by the time they are released. If you attempt to release a qubit in any state other than $|0\rangle$, it will result in a runtime error. We will see why it is important later, when we look at multi-qubit systems.

## Visualizing Quantum State

Before we continue, let's learn some techniques to visualize the Quantum state of our qubits.

### Display the Quantum State of a Single-Qubit Program

Let's start with a simple scenario: a program that acts on a single qubit. 
The state of the quantum system used by this program can be represented as a complex vector of length 2, or, using Dirac notation,

$$\begin{bmatrix} \alpha \\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle$$

If this program runs on a physical quantum system, there is no way to get the information about the values of $\alpha$ and $\beta$ at a certain point of the program execution from a single observation. 
You would need to run the program repeatedly up to this point, perform a measurement on the system, and aggregate the results of multiple measurements to estimate $\alpha$ and $\beta$.

However, at the early stages of quantum program development the program typically runs on a simulator - a classical program which simulates the behavior of a small quantum system while having complete information about its internal state. 
You can take advantage of this to do some non-physical things, such as peeking at the internals of the quantum system to observe its exact state without disturbing it!

The [DumpMachine](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.diagnostics.dumpmachine) function from the [Microsoft.Quantum.Diagnostics namespace](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.diagnostics) allows you to do exactly that. The output of `DumpMachine` is accurate up to a global phase; sometimes you'll see that all amplitudes are multiplied by some complex number compared to the state you're expecting.

### Demo: DumpMachine For Single-Qubit Systems

The following demo shows how to allocate a qubit and examine its state in Q# using [`DumpMachine`](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.diagnostics.dumpmachine) to output the state of the system at any point in the program without affecting the state.

> Note that the Q# code doesn't have access to the output of `DumpMachine`, so you cannot write any non-physical code in Q#!

@[example]({"id": "single_qubit_dump_machine_demo", "codePath": "./examples/SingleQubitDumpMachineDemo.qs"})

The exact behavior of this function depends on the quantum simulator or processor you are using.

On the simulator used in these demos, this function prints the information on each basis state that has a non-zero amplitude, one basis state per row.
This includes information about the amplitude of the state, the probability of measuring that state, and the phase of the state (more on that later)

Note that each row has the following format:

<table>
    <thead>
        <tr>
            <th>Basis State<br>(|𝜓ₙ…𝜓₁⟩)</th>
            <th>Amplitude</th>
            <th>Measurement Probability</th>
            <th>Phase</th>
        </tr>
    </thead>
</table>

For example, the state $|0\rangle$ would be represented as follows:

<table>
    <tbody>
        <tr>
            <td>|0⟩</td>
            <td>1.0000+0.0000𝑖</td>
            <td>100.0000%</td>
            <td>↑ 0.0000</td></tr>
    </tbody>
</table>

> It is important to note that although we reason about quantum systems in terms of their state, Q# does not have any representation of the quantum state in the language. Instead, state is an internal property of the quantum system, modified using gates. For more information, see [Q# documentation on quantum states](https://docs.microsoft.com/azure/quantum/concepts-dirac-notation#q-gate-sequences-equivalent-to-quantum-states).

@[exercise]({
    "id": "learn_single_qubit_state",
    "title": "Learn the State of a Single Qubit",
    "descriptionPath": "./learn_single_qubit_state/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./learn_single_qubit_state/Verification.qs"
    ],
    "placeholderSourcePath": "./learn_single_qubit_state/Placeholder.qs",
    "solutionPath": "./learn_single_qubit_state/solution.md"
})

@[section]({
    "id": "qubit_visualizing_multi_qubit",
    "title": "Display the Quantum State of a Multi-Qubit Program"
})

Now let's take a look at the general case: a program that acts on $N$ qubits. 
The state of the quantum system used by this program can be represented as a complex vector of length $2^N$, or, using Dirac notation,

$$\begin{bmatrix} x_0 \\\ x_1 \\\ \vdots \\\ x_{2^N-1}\end{bmatrix} = \sum_{k = 0}^{2^N-1} x_k |k\rangle$$

Same as in the single-qubit case, `DumpMachine` allows you to see the amplitudes $x_k$ for all basis states $|k\rangle$ directly.

> Note the use of an integer in the ket notation instead of a bit string with one bit per qubit. 
`DumpMachine` uses big-endian to convert bit strings to integers in the ket notation.
We will learn more details on endianness in the "Multi-Qubit Systems" Kata.

## Demo: DumpMachine for Multi-Qubit Systems

@[example]({"id": "multi_qubit_dump_machine_demo", "codePath": "./examples/MultiQubitDumpMachineDemo.qs"})

@[exercise]({
    "id": "learn_basis_state_amplitudes",
    "title": "Learn Basis State Amplitudes",
    "descriptionPath": "./learn_basis_state_amplitudes/index.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./learn_basis_state_amplitudes/Verification.qs"
    ],
    "placeholderSourcePath": "./learn_basis_state_amplitudes/Placeholder.qs",
    "solutionPath": "./learn_basis_state_amplitudes/solution.md"
})

@[section]({
    "id": "qubit_conclusion",
    "title": "Conclusion"
})

This should be enough for you to gain a basic understanding of qubits and qubit states. Next, you will learn how to manipulate those states in the "Single-Qubit Gates" Kata.
