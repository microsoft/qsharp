# Phase Estimation

@[section]({
    "id": "phase_estimation__overview",
    "title": "Overview"
})

This kata introduces you to the phase estimation algorithm - an important building block in more advanced quantum algorithms such as integer factoring.

**This kata covers the following topics:**

- The definition of eigenvalues and eigenvectors
- The phase estimation problem
- The quantum phase estimation algorithm based on quantum Fourier transform

**What you should know to start working on this kata:**

- Basic quantum gates and measurements.
- Quantum Fourier transform.

@[section]({
    "id": "phase_estimation__eigen",
    "title": "Eigenvectors, Eigenvalues, and Eigenphases"
})

An *eigenvector* of a matrix $A$ is a non-zero vector that, when multiplied by that matrix, changes by a scalar factor:

$$A \ket{v} = \lambda \ket{v}$$

The number $\lambda$ is called an *eigenvalue* that corresponds to this eigenvector. In general, eigenvalues of matrices can be complex numbers.

Recall that all quantum gates are unitary matrices, for which their inverse equals their adjoint ($U^{-1} = U^\dagger$). This means that the eigenvalues of their eigenvectors have the property that their modulus equals $1$: 

$$|\lambda| = 1$$

Thus, they can be written in the following form:
$$\lambda = e^{i\theta}$$

The value $\theta$ is called an *eigenphase* that corresponds to this eigenvector.

> How can you prove that the modulus of an eigenvalue of a unitary matrix equals $1$?
> 
> On one hand, by definition of an eigenvalue, 
> $$U \ket{v} = \lambda \ket{v}$$
> $$|U \ket{v}| = |\lambda| \cdot |\ket{v}|$$
> On the other hand, using the properties of the unitary matrix, you can write the following equation:
> $$|U \ket{v}|^2 = \bra{v} U^\dagger U \ket{v} = \bra{v} U^{-1} U \ket{v} = \bra{v} I \ket{v} = \braket{v|v} = |\ket{v}|^2$$
>
> From these two equations, you get the following equality:
> $$(|\lambda| \cdot |\ket{v}|)^2 = |\ket{v}|^2$$
> And then, finally:
> $$|\lambda| = 1$$

If the quantum gate is self-adjoint, that is, its matrix equals its inverse $U^{-1} = U$, the eigenvalues of this matrix can only be $+1$ and $-1$, with eigenphases $0$ and $\pi$, respectively.

> You can prove this in a similar manner, using the defintion of an eigenvalue:
> $$U^2 \ket{v} = U(U \ket{v}) = U(\lambda \ket{v}) = \lambda U \ket{v} = \lambda^2 \ket{v}$$
> At the same time,
> $$U^2 \ket{v} = UU \ket{v} = U^{-1}U \ket{v} = I \ket{v} = \ket{v}$$
> So you can conclude that $\lambda^2 = 1$.

For example, the $Z$ gate has two eigenvctors:
- $\ket{0}$, with eigenvalue $1$
- $\ket{1}$, with eigenvalue $-1$


@[exercise]({
    "id": "phase_estimation__eigenvalues_s",
    "title": "Find Eigenvalues of the S Gate",
    "path": "./eigenvalues_s/"
})

@[exercise]({
    "id": "phase_estimation__eigenvectors_x",
    "title": "Find Eigenvectors of the X Gate",
    "path": "./eigenvectors_x/"
})

@[exercise]({
    "id": "phase_estimation__state_eigenvector",
    "title": "Is Given State an Eigenvector of the Gate?",
    "path": "./state_eigenvector/"
})


@[section]({
    "id": "phase_estimation__problem",
    "title": "Phase Estimation Problem"
})

The phase estimation problem is formulated as follows. 

You are given a unitary operator $U$ and its eigenvector $\ket{\psi}$. The eigenvector is given as a unitary operator $P$ that, when applied to $\ket{0}$, results in the state $\ket{\psi}$.

Your goal is to find the eigenvalue $\lambda$ associated with this eigenvector, or, in a more common formulation, the corresponding eigenphase $\theta$:

$$U\ket{\psi} = e^{2 \pi i \theta} \ket{\psi}, \theta = ?$$

The value of $\theta$ is defined to be between $0$ and $1$, since any value outside of this range has an equivalent value within it. Instead of representing $\theta$ as a decimal, sometimes it is represented as a binary fraction with $n$ digits:

$$\theta = 0.\theta_1 \theta_2... \theta_n = \frac{\theta_1}{2^1}+ \frac{\theta_2}{2^2}+...\frac{\theta_n}{2^n}$$

Let's consider a simplified variant of the phase estimation problem, in which you are guaranteed that the phase $\theta$ has exactly one binary digit, that is, it's either $0$ or $\frac12$.

@[exercise]({
    "id": "phase_estimation__one_bit_eigenphase",
    "title": "One-Bit Phase Estimation",
    "path": "./one_bit_eigenphase/"
})


@[section]({
    "id": "phase_estimation__qpe",
    "title": "Quantum Phase Estimation Algorithm"
})

Quantum phase estimation algorithm is the most common algorithm used for estimating the eigenphase that corresponds to the given unitary-eigenvector pair.
It relies on quantum Fourier transform, and allows you to estimate $n$ binary digits of the eigenphase $\theta$.
Let's see how this algorithm works.

### Inputs

1. A single-qubit unitary $U$ that acts on $m$ qubits.
2. An $m$-qubit state $\ket{\psi}$ - an eigenvector of $U$.
3. An integer $n$ that specifies the desired binary precision of the eigenphase estimate.

### The starting state

The algorithm acts on $n+m$ qubits, split in two registers.
- The first $n$ qubits start in the state $\ket{0...0} = \ket{0}^{\otimes N}$. These qubits are used to get the eigenvalue.
- The last $m$ qubits start in the state $\ket{\psi}$ - the eigenvector of $U$.

### Step 1. Apply Hadamard transform to each qubit of the first register

This step prepares the first qubit in an even superposition of all $n$-qubit basis states, or, equivalently, in an even superposition of all integers from $\ket{0}$ to $\ket{2^n-1}$.

The state of the system will end up being

$$\frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1} \ket{k} \otimes \ket{\psi}$$


### Step 2. Apply the controlled $U$ ladder

On this step, the algorithm applies a series of controlled $U$ gates, with different qubits of the first register as controls and the second register as the target.

Let's assume that the first register stores integers in little-endian notation: the first qubit corresponds to the least-significant bit, and the last qubit - to the most-significant bit. Then, the sequence of gates applied on this step looks as follows:

1. Controlled $U$ gate with the first qubit as control.
2. Controlled $U^2$ gate with the second qubit as control.
3. Controlled $U^4$ gate with the third qubit as control.
4. And so on, ending with controlled $U^{2^{n-1}}$ gate with qubit $n$ as control.

What does this sequence of gates do?

Let's consider the scenario in which the first register is in the basis state $\ket{k} = \ket{k_0} \otimes \ket{k_1} \otimes ... \otimes \ket{k_{n-1}}$, where $k$ is an integer with $n$ bits in its binary notation: 
$$k = 2^0k_0 + 2^1 k_1 + 2^2k_2 + ... + 2^{n-1}k_{n-1}$$

In this case, this sequence of gates will apply the unitary $U^k$ to the second register! Indeed, 

1. Controlled $U$ gate with the first qubit as control will apply $U$ gate if $k_0 = 1$, and do nothing if $k_0 = 0$.
2. Controlled $U^2$ gate with the second qubit as control will apply $U^2$ gate if $k_1 = 1$, and do nothing if $k_1 = 0$.
3. Controlled $U^4$ gate with the third qubit as control will apply $U^4$ gate if $k_2 = 1$, and do nothing if $k_2 = 0$.
4. And so on.

You can see that in the end all the powers of $U$ applied to the second register add up to exactly the number $k$ written in the first register.

If the first register is in a superposition of states, as is the case for this algorithm, you can write the effect of controlled gates on the state of the system using linearity of unitary transformations:

$$\frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1} \ket{k} \otimes \ket{\psi}\rightarrow \frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1} \ket{k} \otimes U^k\ket{\psi}$$

Now, recall that $\ket{\psi}$ is an eigenvector of $U$: 
$$U\ket{\psi} = e^{2 \pi i \theta} \ket{\psi}$$

This means that
$$U^k\ket{\psi} = (e^{2 \pi i \theta})^k \ket{\psi} = e^{2 \pi i \theta \cdot k} \ket{\psi}$$

You can use this to write the following expression for the state of the system after this step:

$$\frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1} e^{2 \pi i \theta \cdot k} \ket{k} \otimes \ket{\psi}$$

Notice that the first and second registers end up not being entangled with each other.


### Step 3. Apply inverse QFT

Now, recall that quantum Fourier transform that acts on a basis state $\ket{j}$ that represents an $n$-bit integer $j$ performs the following transformation:

$$\ket{j} \rightarrow \frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1}  e^{2\pi i jk/2^n} \ket{k} = \frac1{\sqrt{2^n}}\sum_{k=0}^{2^n-1}  e^{2\pi i (j/2^n) \cdot k} \ket{k}$$

If you compare this expression with the state of the first register at the end of the previous step, you'll notice that they match exactly, with $\theta = j / {2^n}$.

This means that you can use inverse Fourier transform (the adjoint of the Fourier transform) to convert the state of the first register to a binary notation of the integer that stores the first $n$ bits of $\theta$ written as a binary fraction.
If $\theta = 0.\theta_1 \theta_2... \theta_n$, applying inverse QFT to the first register will yield a state 

$$\ket{j} = \ket{\theta_n} \otimes ... \otimes \ket{\theta_1}$$

(Remember that the previous step assumed that the first register stores the integers in little-endian notation, so you have to use the same notation on this step.)

### Step 4. Measure the first register

On the last step, you measure the first register, convert the measurement results to an integer, and divide it by $2^n$ to get the eigenphase $\theta$.

@[exercise]({
    "id": "phase_estimation__implement_qpe",
    "title": "Implement QPE Algorithm",
    "path": "./implement_qpe/"
})


@[section]({
    "id": "phase_estimation__practicality",
    "title": "QPE Algorithm: Practical Considerations"
})

Now that you've learned to implement the QPE algorithm, let's discuss several practical aspects of its applications.

### Applying QPE to a superposition of eigenvectors

First of all, the algorithm inputs include an eigenvector of the unitary for which you want to estimate the eigenphase. In some applications, the eigenvector is either unknown or hard to prepare accurately. Is there a workaround for these scenarios?

To answer this, let's see what happens if you run the phase estimation algorithm for a state $\ket{\phi}$ that is not an eigenstate of the unitary $U$.

For any unitary $U$, you can find a basis that consists of its eigenvectors. This means that any state $\ket{\phi}$ can be represented as a superposition of eigenvectors $\ket{\psi_k}$: 
$$\ket{\phi} = \sum_k a_k\ket{\psi_k}$$

You can notice that all the steps of the QPE algorithm until the final measurement are linear transformations. This means that you can apply them to each term in the superposition independently.

The state of the system right before the final measurement will look as follows:

$$\sum_k a_k \ket{j_k} \ket{\psi_k}$$

Here $j_k$ is the integer representation of the $n$ binary digits of the eigenphase that corresponds to the eigenvector $\ket{\psi_k}$. 

Now, when you measure the first register, this measurement will cause the collapse of the superposition: you'll get one of the values $j_k$ that describes one of the eigenphases, and the second register will collapse to the matching eigenvector.

This means that if you're looking for *any* eigenphase of the unitary, you can just run the QPE algorithm with any state in place of the eigenvector. And if you know the eigenvector you want to use but it's tricky to prepare precisely, you can prepare it approximately and run the QPE algorithm on this state to get the eigenphase associated with it with high probability.


### Eigenphases that are not $n$-bit binary fractions

The algorithm derivation in the previous lesson assumed that the eigenphase $\theta$ can be written down with exactly $n$ binary bits of precision. However, this is not always the case. What happens if the eigenphase of the given eigenvector can't be written down using exactly $n$ binary bits?

It turns out that the QPE algorithm running with $n$ bits of precision is deterministic only for eigenphases that can be represented precisely as $n$-bit binary fractions. Running it for any eigenphases that don't have such a representation yields probabilistic results. The outcomes that are the closest $n$-bit fractions to the true eigenphase are produced with the highest probability, but it's also possible to get outcomes that are pretty far away from the true eigenphase.

The following demo allows you to play with the QPE algorithm for different eigenphases and to observe its behavior in different scenarios.
For example, you can see that running phase estimation for the gate `R1Frac(1, 3, _)` which has eigenphase $\frac1{16} = 0.0625$
with $n = 3$ bits of precision yields non-deterministic results, with the two most frequent outcomes being $0$ and $\frac18 = 0.125$.

@[example]({"id": "phase_estimation__e2edemo", "codePath": "./examples/QuantumPhaseEstimationDemo.qs"})


@[section]({
    "id": "phase_estimation__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned about the phase estimation problem and its solution using the quantum phase estimation algorithm.
