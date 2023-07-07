# Quantum Random Number Generation Tutorial

True random number generation is a notoriously difficult problem. Many "random" generators today are actually pseudo-random, using a starting seed to spawning seemingly-random numbers that are actually a repeatable function of that seed. Most true random number generations are based on measurements of some natural phenomenon, such as atmospheric noise or atomic decay. 
(You can read more about it [here]( https://en.wikipedia.org/wiki/Random_number_generation).) 

Quantum random number generators (QRNGs) are truly random. The quantum algorithm for random number generation is one of the simplest applications of quantum computing principles, requiring very few qubits to run.

**In this tutorial you will:**
* learn about quantum random number generation and the principles behind it,
* implement a variety of QRNGs with equal probability of any given number,
* implement a single-bit QRNG with weighted probabilities of generated bits.

**What you should know for this workbook**

You should be familiar with the following concepts before tackling the Quantum Random Number Generation Tutorial (and this workbook):

1. The concept of qubit and measurement
2. Single-qubit gates

Let's go!

## Introduction

Recall from the [Qubit](../Qubit/Qubit.ipynb) tutorial that a qubit state $|\psi\rangle$ is defined via the basis states $|0\rangle$ and $|1\rangle$ as:

$$|\psi\rangle = \begin{bmatrix} \alpha \\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle\text{, where }|\alpha|^2 + |\beta|^2 = 1$$

We call $\alpha$ and $\beta$ the **amplitudes** of states $|0\rangle$ and $|1\rangle$, respectively. 
When $|\psi\rangle$ is measured in the $\{|0\rangle, |1\rangle\}$ basis (the computational basis), the probabilities of the outcomes are defined based on the state amplitudes: there is a $|\alpha|^2$ probability that the measurement result will be $0$, and a $|\beta|^2$ probability that the measurement result will be $1$.

> For example, a qubit in state $\begin{bmatrix} \frac{1}{\sqrt{2}} \\ \frac{1}{\sqrt{2}} \end{bmatrix}$ will yield measurement results $0$ or $1$ with equal probability, while a qubit in state $\begin{bmatrix} \frac{1}{2} \\ \frac{\sqrt3}{2} \end{bmatrix}$ will yield measurement result $0$ only 25% of the time, and $1$ 75% of the time.

This is sufficient to implement a simple random number generator!

> Remember that you can refer to the [Single Qubit Gates tutorial](../SingleQubitGates/SingleQubitGates.ipynb) if you need a refresher on the various quantum gates and their usage in Q#.

## <span style="color:blue">Exercise 1</span>: Generate a single random bit

**Input:** None.

**Goal:** Generate a $0$ or $1$ with equal probability.

<details>
    <summary><strong>Need a hint? Click here</strong></summary>
    Use the allocated qubit, apply a quantum gate to it, measure it and use the result to return a $0$ or $1$.
</details>

**Stretch goal:** Can you find a different way to implement this operation?

<details>
    <summary><strong>Need a hint? Click here</strong></summary>
    What are the different quantum states that produce $0$ and $1$ measurement results with the same probability? How would measuring the qubit in a different basis change the result? 
</details>

@[exercise]({
"id": "random_bit",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
],
"verificationSourcePath": "./random_bit/verification.qs",
"placeholderSourcePath": "./random_bit/placeholder.qs",
"solutionSourcePath": "./random_bit/solution.qs",
"solutionDescriptionPath": "./random_bit/solution.md"
})

## <span style="color:blue">Exercise 2</span>: Generate a random two-bit number

Now that you can generate a single random bit, you can use that logic to create random multi-bit numbers. Let's try first to make a two-bit number by combining two randomly generated bits.

**Input:** None.

**Goal:** Generate a random number in the range $[0, 3]$ with an equal probability of getting each of the four numbers.

**Stretch goal:** Can you do this without allocating qubits in this operation?

<details>
    <summary><strong>Need a hint? Click here</strong></summary>
    Remember that you can use the previously defined operations.
</details>

@[exercise]({
"id": "random_two_bits",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
],
"verificationSourcePath": "./random_two_bits/verification.qs",
"placeholderSourcePath": "./random_two_bits/placeholder.qs",
"solutionSourcePath": "./random_two_bits/solution.qs",
"solutionDescriptionPath": "./random_two_bits/solution.md"
})

## <span style="color:blue">Exercise 3</span>: Generate a number of arbitrary size

Let's take it a step further and generate an $N$-bit number. 

> Remember that you can use previously defined operations in your solution.

**Input:** An integer $N$ ($1 \le N \le 10$).

**Goal:** Generate a random number in the range $[0, 2^N - 1]$ with an equal probability of getting each of the numbers in this range.

> Useful Q# documentation: 
> * [`for` loops](https://docs.microsoft.com/azure/quantum/user-guide/language/statements/iterations), 
> * [mutable variables](https://docs.microsoft.com/azure/quantum/user-guide/language/typesystem/immutability), 
> * [exponents](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.powi).

@[exercise]({
"id": "random_n_bits",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
],
"verificationSourcePath": "./random_n_bits/verification.qs",
"placeholderSourcePath": "./random_n_bits/placeholder.qs",
"solutionSourcePath": "./random_n_bits/solution.qs",
"solutionDescriptionPath": "./random_n_bits/solution.md"
})

## <span style="color:blue">Exercise 4</span>: Generate a weighted bit!

In each of the above exercises, all generated numbers were equally likely. Now let's create an operation that will return a random bit with different probabilities of outcomes. 

> Remember that by setting amplitudes of basis states $\alpha$ and $\beta$, we can control the probability of getting measurement outcomes $0$ and $1$ when the qubit is measured.

**Input:** 
A floating-point number $x$, $0 \le x \le 1$. 

**Goal:** Generate $0$ or $1$ with probability of $0$ equal to $x$ and probability of $1$ equal to $1 - x$.

> Useful Q# documentation: 
> * [`Math` namespace](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math)
> * [`ArcCos` function](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arccos)
> * [`Sqrt` function](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.sqrt)

@[exercise]({
"id": "weighted_random_bit",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
],
"verificationSourcePath": "./weighted_random_bit/verification.qs",
"placeholderSourcePath": "./weighted_random_bit/placeholder.qs",
"solutionSourcePath": "./weighted_random_bit/solution.qs",
"solutionDescriptionPath": "./weighted_random_bit/solution.md"
})

## <span style="color:blue">Exercise 5</span>: Generate a random number between min and max

In exercise 3, we generated numbers in the range $[0, 2^N-1]$ $(1 \leq N \leq 10)$. Now let's create an operation that will return a random number in the range $[min, max]$. 

**Input:** 
Two integers $min$ and $max$ ($0 \leq min \leq max \leq 2^{10}-1$).

**Goal:** Generate a random number in the range $[min, max]$ with an equal probability of getting each of the numbers in this range.

> Useful Q# documentation: 
> * [`BitSizeI` function](https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.math.bitsizei)

@[exercise]({
"id": "random_number",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
],
"verificationSourcePath": "./random_number/verification.qs",
"placeholderSourcePath": "./random_number/placeholder.qs",
"solutionSourcePath": "./random_number/solution.qs",
"solutionDescriptionPath": "./random_number/solution.md"
})

## What's Next?
We hope you enjoyed this tutorial on quantum random number generation! If you're looking to learn more about quantum computing and Q#, here are some suggestions:
* The [Quantum Katas](https://github.com/microsoft/QuantumKatas/) are sets of programming exercises on quantum computing that can be solved using Q#. They cover a variety of topics, from the basics like the concepts of superposition and measurements to more interesting algorithms like Grover's search.
* For another look at quantum random number generation, you can check out the [Microsoft Learn module "Create your first Q# program by using the Quantum Development Kit"](https://docs.microsoft.com/learn/modules/qsharp-create-first-quantum-development-kit/1-introduction).
