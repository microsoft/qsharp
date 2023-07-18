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

@[exercise]({
"id": "random_bit",
"descriptionPath": "./random_bit/index.md",
"placeholderSourcePath": "./random_bit/placeholder.qs",
"verificationSourcePath": "./random_bit/verification.qs",
"solutionPath": "./random_bit/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "random_two_bits",
"descriptionPath": "./random_two_bits/index.md",
"placeholderSourcePath": "./random_two_bits/placeholder.qs",
"verificationSourcePath": "./random_two_bits/verification.qs",
"solutionPath": "./random_two_bits/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "random_n_bits",
"descriptionPath": "./random_n_bits/index.md",
"placeholderSourcePath": "./random_n_bits/placeholder.qs",
"verificationSourcePath": "./random_n_bits/verification.qs",
"solutionSourcePath": "./random_n_bits/solution.qs",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "weighted_random_bit",
"descriptionPath": "./weighted_random_bit/index.md",
"placeholderSourcePath": "./weighted_random_bit/placeholder.qs",
"verificationSourcePath": "./weighted_random_bit/verification.qs",
"solutionSourcePath": "./weighted_random_bit/solution.qs",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "random_number",
"descriptionPath": "./random_number/index.md",
"placeholderSourcePath": "./random_number/placeholder.qs",
"verificationSourcePath": "./random_number/verification.qs",
"solutionSourcePath": "./random_number/solution.qs",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

## What's Next?
We hope you enjoyed this tutorial on quantum random number generation! If you're looking to learn more about quantum computing and Q#, here are some suggestions:
* The [Quantum Katas](https://github.com/microsoft/QuantumKatas/) are sets of programming exercises on quantum computing that can be solved using Q#. They cover a variety of topics, from the basics like the concepts of superposition and measurements to more interesting algorithms like Grover's search.
* For another look at quantum random number generation, you can check out the [Microsoft Learn module "Create your first Q# program by using the Quantum Development Kit"](https://docs.microsoft.com/learn/modules/qsharp-create-first-quantum-development-kit/1-introduction).
