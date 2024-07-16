# Quantum Random Number Generation

@[section]({"id": "random_numbers__overview", "title": "Overview"})

True random number generation is a notoriously difficult problem. Many "random" generators today are actually pseudo-random, using a starting seed to spawn seemingly-random numbers that are actually a repeatable function of that seed. Most true random number generators are based on measurements of some natural phenomenon, such as atmospheric noise or atomic decay. You can read more about it <a href="https://en.wikipedia.org/wiki/Random_number_generation" target="_blank">here</a>.

Quantum random number generators (QRNGs) are truly random. Of course, this only applies to the case when they run on a real quantum device, relying on the randomness of the quantum state collapse during measurement to produce the random numbers. When QRNGs run on a simulator, the source of randomness is the same as for other classical programs, so the generated numbers are pseudo-random.

The quantum algorithm for random number generation is one of the simplest applications of quantum computing principles, requiring very few qubits to run.

**This kata covers the following topics:**

- Quantum random number generation and the principles behind it
- Implementation of a variety of QRNGs with equal probability of any given number
- Implementation a single-bit QRNG with weighted probabilities of generated bits

**What you should know to start working on this kata:**

- The concept of qubit and measurement
- Single-qubit gates

@[section]({"id": "random_numbers__introduction", "title": "Introduction"})

Recall from the Qubit kata that a qubit state $\ket{\psi}$ is defined via the basis states $\ket{0}$ and $\ket{1}$ as $\ket{\psi} = \begin{bmatrix} \alpha \\ \beta \end{bmatrix} = \alpha\ket{0} + \beta\ket{1}$, where $|\alpha|^2 + |\beta|^2 = 1$.

We call $\alpha$ and $\beta$ the probability amplitudes of states $\ket{0}$ and $\ket{1}$, respectively. When $\ket{\psi}$ is measured in the $\{\ket{0}, \ket{1}\}$ basis (the computational basis), the probabilities of the outcomes are defined based on the state amplitudes: there is a $|\alpha|^2$ probability that the measurement result will be $0$, and a $|\beta|^2$ probability that the measurement result will be $1$.

> For example, a qubit in state $\begin{bmatrix} \frac{1}{\sqrt{2}} \\ \frac{1}{\sqrt{2}} \end{bmatrix}$ will yield measurement results $0$ or $1$ with equal probability, while a qubit in state $\begin{bmatrix} \frac{1}{2} \\ \frac{\sqrt3}{2} \end{bmatrix}$ will yield measurement result $0$ only 25% of the time, and $1$ 75% of the time.

This knowledge is sufficient to implement a simple random number generator!

> Remember that you can refer to the Single-Qubit Gates kata if you need a refresher on the various quantum gates and their usage in Q#.

@[exercise]({
    "id": "random_numbers__random_bit",
    "title": "Generate a Single Random Bit",
    "path": "./random_bit/"
})

@[exercise]({
    "id": "random_numbers__random_two_bits",
    "title": "Generate a Random Two-Bit Number",
    "path": "./random_two_bits/"
})

@[exercise]({
    "id": "random_numbers__random_n_bits",
    "title": "Generate a Number of Arbitrary Size",
    "path": "./random_n_bits/"
})

@[exercise]({
    "id": "random_numbers__weighted_random_bit",
    "title": "Generate a Weighted Bit",
    "path": "./weighted_random_bit/"
})

@[exercise]({
    "id": "random_numbers__random_number",
    "title": "Generate a Random Number Between Min and Max",
    "path": "./random_number/"
})

@[section]({"id": "random_numbers__whats_next", "title": "What's Next?"})

Congratulations! In this kata you have created a random number generator. Here are a few key concepts to keep in mind:

- This code will generate truly random numbers when executed on a true quantum computer. Random numbers obtained when executing on a simulator are only as good as the source of randomness used by the simulator.
- You can generate a random bit by preparing a qubit in superposition and then measuring it in the computational basis.
  The amplitudes of the basis states will define the probability distribution of the generated bits.
- The Q# library function `BitSizeI` returns the number of bits in the binary representation of an integer.
