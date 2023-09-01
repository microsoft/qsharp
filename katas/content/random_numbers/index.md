# Quantum Random Number Generation

@[section]({"id": "random_numbers__overview", "title": "Overview"})

True random number generation is a notoriously difficult problem. Many "random" generators today are actually pseudo-random, using a starting seed to spawn seemingly-random numbers that are actually a repeatable function of that seed. Most true random number generators are based on measurements of some natural phenomenon, such as atmospheric noise or atomic decay. You can read more about it <a href="https://en.wikipedia.org/wiki/Random_number_generation" target="_blank">here</a>. 

Quantum random number generators (QRNGs) are truly random. The quantum algorithm for random number generation is one of the simplest applications of quantum computing principles, requiring very few qubits to run.

**This kata covers the following topics:**

- Quantum random number generation and the principles behind it
- Implementation of a variety of QRNGs with equal probability of any given number
- Implementation a single-bit QRNG with weighted probabilities of generated bits

**What you should know to start working on this kata:**

- The concept of qubit and measurement
- Single-qubit gates

@[section]({"id": "random_numbers__introduction", "title": "Introduction"})

Recall from the Qubit kata that a qubit state $|\psi\rangle$ is defined via the basis states $|0\rangle$ and $|1\rangle$ as $|\psi\rangle = \begin{bmatrix} \alpha \\ \beta \end{bmatrix} = \alpha|0\rangle + \beta|1\rangle$, where $|\alpha|^2 + |\beta|^2 = 1$

We call $\alpha$ and $\beta$ the probability amplitudes of states $|0\rangle$ and $|1\rangle$, respectively. When $|\psi\rangle$ is measured in the $\\{|0\rangle, |1\rangle\\}$ basis (the computational basis), the probabilities of the outcomes are defined based on the state amplitudes: there is a $|\alpha|^2$ probability that the measurement result will be $0$, and a $|\beta|^2$ probability that the measurement result will be $1$.

> For example, a qubit in state $\begin{bmatrix} \frac{1}{\sqrt{2}} \\\ \frac{1}{\sqrt{2}} \end{bmatrix}$ will yield measurement results $0$ or $1$ with equal probability, while a qubit in state $\begin{bmatrix} \frac{1}{2} \\\ \frac{\sqrt3}{2} \end{bmatrix}$ will yield measurement result $0$ only 25% of the time, and $1$ 75% of the time.

This knowledge is sufficient to implement a simple random number generator!

> Remember that you can refer to the Single-Qubit Gates kata if you need a refresher on the various quantum gates and their usage in Q#.

@[exercise]({
    "id": "random_numbers__random_bit",
    "title": "Generate a Single Random Bit",
    "descriptionPath": "./random_bit/index.md",
    "placeholderSourcePath": "./random_bit/placeholder.qs",
    "solutionPath": "./random_bit/solution.md",
    "codePaths": [
        "./random_bit/verification.qs",
        "../KatasLibrary.qs",
        "./common.qs"
    ]
})

@[exercise]({
    "id": "random_numbers__random_two_bits",
    "title": "Generate a Random Two-Bit Number",
    "descriptionPath": "./random_two_bits/index.md",
    "placeholderSourcePath": "./random_two_bits/placeholder.qs",
    "solutionPath": "./random_two_bits/solution.md",
    "codePaths": [
        "./random_two_bits/verification.qs",
        "../KatasLibrary.qs",
        "./common.qs"
    ]
})

@[exercise]({
    "id": "random_numbers__random_n_bits",
    "title": "Generate a Number of Arbitrary Size",
    "descriptionPath": "./random_n_bits/index.md",
    "placeholderSourcePath": "./random_n_bits/placeholder.qs",
    "solutionPath": "./random_n_bits/solution.md",
    "codePaths": [
        "./random_n_bits/verification.qs",
        "../KatasLibrary.qs",
        "./common.qs"
    ]
})

@[exercise]({
    "id": "random_numbers__weighted_random_bit",
    "title": "Generate a Weighted Bit",
    "descriptionPath": "./weighted_random_bit/index.md",
    "placeholderSourcePath": "./weighted_random_bit/placeholder.qs",
    "solutionPath": "./weighted_random_bit/solution.md",
    "codePaths": [
        "./weighted_random_bit/verification.qs",
        "../KatasLibrary.qs",
        "./common.qs"
    ]
})

@[exercise]({
    "id": "random_numbers__random_number",
    "title": "Generate a Random Number Between Min and Max",
    "descriptionPath": "./random_number/index.md",
    "placeholderSourcePath": "./random_number/placeholder.qs",
    "solutionPath": "./random_number/solution.md",
    "codePaths": [
        "./random_number/verification.qs",
        "../KatasLibrary.qs",
        "./common.qs"
    ]
})

@[section]({"id": "random_numbers__whats_next", "title": "What's Next?"})

Congratulations! In this Kata you have created a truly random number generator using quantum computing. Here are a few key concepts to keep in mind:
* You can generate a random bit by applying a Hadamard gate to a state $\ket{0}$, and then measuring the resulting qubit in the computational basis.
* The Q# <a href="https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.math.bitsizei" target="_blank">BitSizeI function</a> returns the numbers of bits needed to write an integer in binary.

**Next Steps**

We hope you enjoyed this Kata on quantum random number generation! If you're looking to learn more about quantum computing and Q#, here are some suggestions:
* To learn about superposition, interference and entanglement by using Q#, you can check <a href="https://learn.microsoft.com/en-us/training/modules/qsharp-explore-key-concepts-quantum-computing/" target="_blank">Microsoft Learn module "Explore the key concepts of quantum computing by using Q#"</a>.
* For another look at quantum random number generation, you can check out the <a href="https://docs.microsoft.com/learn/modules/qsharp-create-first-quantum-development-kit/1-introduction" target="_blank">Microsoft Learn module "Create your first Q# program by using the Quantum Development Kit"</a>.

