# Quantum Fourier Transform

@[section]({
    "id": "qft__overview",
    "title": "Overview"
})

This kata introduces you to quantum Fourier transform (QFT) - an important tool in quantum phase estimation, integer factoring, and many other quantum computing algorithms.

**This kata covers the following topics:**

- The definition of quantum Fourier transform
- The iterative implementation of QFT
- Simple state preparation and state analysis tasks that can be solved using QFT

**What you should know to start working on this kata:**

- Basic knowledge of quantum states and quantum gates.

If you need a refresher on these topics, you can check out the previous katas.


@[section]({
    "id": "qft__implement",
    "title": "Implementing Quantum Fourier Transform"
})

*Discrete Fourier transform* (DFT) is a transform that acts on vectors of complex numbers of fixed length $N$. The effect of DFT on a vector $x_0, x_1, ..., x_{N-1}$ is another vector $y_0, y_1, ..., y_{N-1}$ defined as follows:

$$y_k = \frac1{\sqrt{N}}\sum_{j=0}^{N-1} x_j e^{2\pi i jk/N}$$

Quantum Fourier transform is the quantum version of the DFT. It acts on $n$-qubit quantum states and is defined via its effects on basis states as follows:

$$\ket{j} \rightarrow \frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1}  e^{2\pi i jk/2^n} \ket{k}$$

The result of applying QFT to an arbitrary superposition state with amplitudes $x_j$ can thus be expressed as a superposition state with amplitudes $y_k$ that are exactly the DFT of the amplitudes $x_j$ for $N = 2^n$:

$$\sum_{j=0}^{N-1} x_j\ket{j} \rightarrow \sum_{k=0}^{N-1} y_k\ket{k}$$

In the first part of this kata, you will learn to implement quantum Fourier transform using the iterative algorithm.

All the numbers used in this kata use big endian encoding: the most significant bit of the number is stored in the first (leftmost) bit/qubit.
 
This means that you represent an integer as a binary bit string in the following format:

$$x = x_1x_2...x_n = x_1 2^{n-1} + x_2 2^{n-2}+...x_n 2^{0}$$
 
You can also use this notation for binary fractions: 
 
$$0.x_1x_2...x_n = \frac{x_1}{2^1}+ \frac{x_2}{2^2}+...\frac{x_n}{2^n}$$
 
@[exercise]({
    "id": "qft__single_qubit",
    "title": "Single-Qubit QFT",
    "path": "./single_qubit/"
})

@[exercise]({
    "id": "qft__rotation_gate",
    "title": "Rotation Gate",
    "path": "./rotation_gate/"
})

@[exercise]({
    "id": "qft__binary_fraction_classical",
    "title": "Binary Fraction Exponent (Classical Input)",
    "path": "./binary_fraction_classical/"
})

@[exercise]({
    "id": "qft__binary_fraction_quantum",
    "title": "Binary Fraction Exponent (Quantum Input)",
    "path": "./binary_fraction_quantum/"
})

@[exercise]({
    "id": "qft__binary_fraction_inplace",
    "title": "Binary Fraction Exponent In-Place (Quantum Input)",
    "path": "./binary_fraction_inplace/"
})

@[exercise]({
    "id": "qft__qft",
    "title": "Quantum Fourier Transform",
    "path": "./qft/"
})


@[section]({
    "id": "qft__equivalence",
    "title": "The Equivalence of Two QFT Notations"
})

How to show that the transformation defined in the last exercise is the same as the original definition of QFT?
In other words, how to prove the equality below?

$$\frac1{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i \cdot \frac{jk}{2^{n}}} \ket{k} = $$
$$\begin{align*}
= &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_n} \ket{1} \big) \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_{n-1} j_n} \ket{1} \big) \otimes ... \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot 0.j_1 j_2 ... j_{n-1} j_n} \ket{1} \big)
\end{align*}$$
 
The tensor product state is defined in terms of the digits $j_1, j_2, ..., j_n$. Let's spell the exponents used in each term of the tensor product in terms of the integer $j$ instead:
$$e^{2\pi i \cdot 0.j_1 j_2 ... j_{n-1} j_n} = e^{2\pi i \cdot \frac{j}{2^n}}$$
$$e^{2\pi i \cdot 0.j_2 ... j_{n-1} j_n} = e^{2\pi i \cdot j_1} e^{2\pi i \cdot 0.j_2 ... j_{n-1} j_n} = e^{2\pi i \cdot j_1.j_2 ... j_{n-1} j_n} = e^{2\pi i \cdot \frac{j \cdot 2}{2^n}}$$
(Remember that since $j_1$ can only be $0$ or $1$, $e^{2\pi i \cdot j_1}$ is either $e^0$ or $e^{2\pi i}$, which are both $1$. Any exponents of integer multiples of $e^{2\pi i}$ are $1$ and thus can be ignored.)

And so on, until the last term
$$e^{2\pi i \cdot 0.j_n} = e^{2\pi i \cdot \frac{j \cdot 2^{n-1}}{2^n}}$$

Now, the complete state looks as follows:
$$\begin{align*}
&\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j \cdot 2^{n-1}}{2^n}} \ket{1} \big) \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j \cdot 2^{n-2}}{2^n}} \ket{1} \big) \otimes ... \otimes \\
\otimes &\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j}{2^n}} \ket{1} \big)
\end{align*}$$

What happens if you open the brackets and compare the results to the definition of the QFT?
Let's do this exercise for $n = 2$.

$$\frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j \cdot 2}{4}} \ket{1} \big) \otimes \frac1{\sqrt2} \big(\ket{0} + e^{2\pi i \cdot \frac{j}{4}} \ket{1} \big) = $$
$$= \frac14 \big( \ket{00} + e^{2\pi i \cdot \frac{j}{4}}\ket{01} + e^{2\pi i \cdot \frac{j \cdot 2}{4}}\ket{10} + e^{2\pi i \cdot \frac{j \cdot 2 + j}{4}} \ket{11} \big)$$

Remember that this kata uses big endian to map bit strings to integers, so this can be rewritten as follows:
$$\frac14 \big( \ket{0} + e^{2\pi i \cdot \frac{j}{4}}\ket{1} + e^{2\pi i \cdot \frac{j \cdot 2}{4}}\ket{2} + e^{2\pi i \cdot \frac{j \cdot 3}{4}} \ket{3} \big)$$
And that's exactly the original definition of QFT:
$$\frac1{\sqrt{4}}\sum_{k=0}^{3}  e^{2\pi i jk/4} \ket{k}$$

The same can be done for an arbitrary $n$.


@[section]({
    "id": "qft__use",
    "title": "Using Quantum Fourier Transform"
})

This section offers you tasks on state preparation and state analysis that can be solved using QFT or inverse QFT. 
You can solve them without QFT, but it is useful to try and come up with a QFT-based solution as an exercise.

TODO: finish part 2


@[section]({
    "id": "qft__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to implement quantum Fourier transform and to use it in simple tasks. In the next kata, you will learn to use QFT to solve a more complicated problem - the phase estimation task.
