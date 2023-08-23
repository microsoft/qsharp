# Measurements in Single-Qubit Systems

@[section]({
    "id": "single_qubit_measurements_overview",
    "title": "Overview"
})

This Kata introduces you to measurements done on single-qubit systems.

The concept of a measurement is a central part of quantum mechanics, as well as quantum algorithms. Single-qubit measurements, as their name implies, are measurements on single qubits. The outcomes of a measurement in quantum mechanics are probabilistic, and in general, change the state of the qubit according to the outcome of the measurement.

We recommend you complete the "Single-Qubit Gates" Kata before starting this one.

**This Kata covers the following topics:**

- Computational basis measurements
- Pauli basis measurements
- Measurements in arbitrary orthogonal bases
- Representing measurements as projector operators

**What you should know to start working on this Kata:**

- Basic linear algebra
- The concept of a qubit
- Single-qubit gates

$\renewcommand{\ket}[1]{\left\lvert#1\right\rangle}$
$\renewcommand{\bra}[1]{\left\langle#1\right\rvert}$

@[section]({
    "id": "single_qubit_measurements_overview",
    "title": "Computational Basis Measurements"
})

In this section, we will discuss the simplest type of qubit measurements - measurements in the computational basis. This is the "default" type of measurements. Unless otherwise specified, "measurement" refers to this type.

The state $\ket{\psi}$ of a single qubit can always be expressed in Dirac notation as:
$$\ket{\psi} = \alpha \ket{0} + \beta \ket{1}$$
where $\alpha$ and $\beta$ are complex numbers, and the state is normalized, i.e., $|\alpha|^2 + |\beta|^2 = 1$.

We can examine the qubit to get some information about its state - *measure* its state. Similar to the classical case of examining a bit, the outcome of a measurement can be $0$ or $1$. However, unlike the classical case, quantum measurement is a probabilistic process.

The probabilities of the measurement outcomes being $0$ and $1$ are $|\alpha|^2$ and $|\beta|^2$, respectively. Additionally, the state of the qubit is modified by the measurement - if the outcome of the measurement is $0$, then the post-measurement state of the qubit is $\ket{0}$, and if the outcome is $1$, the state is $\ket{1}$. In quantum mechanics, this is referred to as the [collapse of the wave function](https://en.wikipedia.org/wiki/Wave_function_collapse).

The outcomes of computational basis measurements and their probabilities are summarized in the table below:
<table>
    <tr>
        <th>Measurement outcome</th>
        <th>Probability of outcome</th>
        <th>State after measurement</th>
    </tr>
    <tr>
        <td>$0$</td>
        <td>$|\alpha|^2$</td>
        <td>$\ket 0$</td>
    </tr>
    <tr>
        <td>$1$</td>
        <td>$|\beta|^2$</td>
        <td>$\ket 1$</td>
    </tr>
</table>

>Unlike quantum gates which are unitary and reversible operations, measurements are neither unitary nor reversible. Since the outcomes of a measurement are probabilistic, any two isolated qubits which are initially prepared in identical superposition states are in general not guaranteed to have the same measurement outcomes after each qubit has been measured separately. As we will see below, measurements are modeled by projection operators instead of unitary operators.
>
>Additionally, the assumption of the wave function being **normalized** is important, since the probability outcomes must sum up to $1$. If the wave function is not normalized, it is important to normalize it first in order to obtain the correct measurement probabilities.

## 🔎 Analyze

The qubit is in the following state:
$$\ket \psi = 0.6 \ket 0 + 0.8 \ket 1 \equiv \begin{bmatrix} 0.6 \\\ 0.8\end{bmatrix}.$$

If this qubit is measured in the computational basis, what are the outcome probabilities?

<details>
<summary><b>Solution</b></summary>
The given state $\ket \psi$ is normalized, since $0.6^2 + 0.8^2 = 1$. Hence, the probability of measuring $0$ is $|0.6|^2 = 0.36$, and the probability of measuring $1$ is $|0.8|^2 = 0.64$.
</details>

@[section]({
    "id": "single_qubit_measurements_implementing_measurement",
    "title": "Implementing Measurement In Q# Using The M Operation"
})

In this demo, we prepare a qubit in the state $0.6|0\rangle + 0.8|1\rangle$, and then measure it in the computational basis. In Q#, single-qubit measurements in the computational basis can be implemented using the [M operation](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.m). It will return the constant `Zero` if measurement result was $0$ or the constant `One` if the measurement result was $1$. `Zero` and `One` are constants of type `Result`.

> If you run this code multiple times, you will notice that whenever the measurement outcome is $1$, the post-measurement state of the qubit is $\ket 1$, and similarly for $0$. This is in line with our expectation that after the measurement the wave function 'collapses' to the corresponding state.

@[example]({
    "id": "implementing_measurement",
    "codePath": "./implementing_measurement/example.qs"
})

@[section]({
    "id": "single_qubit_measurements_measurement_statistics",
    "title": "Measurement Statistics"
})

The following code demonstrates that the theoretical and experimental values of the probability outcomes indeed match with each other. We repeatedly prepare the same state $\ket \psi = 0.6 \ket 0 + 0.8 \ket 1$ and measure it in the computational basis $100$ times. At the end, we expect $0$ to be measured approximately $36$ times, and $1$ to be measured approximately $64$ times. Note that since measurements are probabilistic, we do not expect the results to match these values exactly.

@[example]({
    "id": "measurement_statistics",
    "codePath": "./measurement_statistics/example.qs"
})

Measurements can be used to distinguish orthogonal states. We start with an exercise for distinguishing between the computational basis states and discuss the general case of arbitrary basis measurements later in the Kata.

@[exercise]({
    "id": "distinguish_0_and_1",
    "title": "Distinguish |0〉 and |1〉",
    "descriptionPath": "./distinguish_0_and_1/index.md",
    "placeholderSourcePath": "./distinguish_0_and_1/placeholder.qs",
    "solutionPath": "./distinguish_0_and_1/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./distinguish_0_and_1/verification.qs"
    ]
})

@[section]({
    "id": "single_qubit_measurements_pauli_bases",
    "title": "Measurements in the Pauli Bases"
})

So far, we have discussed measurements done in the computational basis, i.e., the $\{ \ket 0, \ket 1\}$ basis.

It is also possible to implement measurements in other orthogonal bases, such as the Pauli X basis, which consists of the two vectors $\ket + = \frac1{\sqrt2} \big(\ket 0 +\ket 1\big)$, and $\ket - = \frac1{\sqrt2} \big(\ket 0 -\ket 1\big)$. Q# has a built-in operation [`Measure`](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure) for measurements in the Pauli bases.

> The `Measure` operation can be used for measuring multiple qubits in a multi-qubit system; however, in this Kata we only consider measurements for single-qubit systems.

The eigenvalues of a Pauli matrix are $\pm 1$, with one eigenvector corresponding to each eigenvalue. For any chosen Pauli basis, the `Measure` operation returns `Zero` if the measurement outcome corresponds to the eigenvalue $+1$, and returns `One` if the measurement outcome corresponds to the eigenvalue $-1$. As in the case of the computational basis measurements, the wave function of the qubit collapses to the corresponding state after the measurement is executed.

The probabilities of the outcomes are defined using a similar rule: to measure a state $\ket \psi$ in a Pauli basis $\{ \ket {b_0}, \ket {b_1}\}$, we represent it as a linear combination of the basis vectors
$$\ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1}.$$

The probabilities of outcomes $0$ and $1$ will be defined as $|c_0|^2$ and $|c_1|^2$, respectively.

> Computational basis measurement is often referred to as measurement in Pauli Z basis. Indeed, the eigenvectors of the Z gate are $\ket 0$ and $\ket 1$, with eigenvalues $+1$ and $-1$, respectively.

@[exercise]({
    "id": "distinguish_plus_and_minus",
    "title": "Distinguish |+〉 and |-〉",
    "descriptionPath": "./distinguish_plus_and_minus/index.md",
    "placeholderSourcePath": "./distinguish_plus_and_minus/placeholder.qs",
    "solutionPath": "./distinguish_plus_and_minus/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./distinguish_plus_and_minus/verification.qs"
    ]
})

@[section]({
    "id": "single_qubit_measurements_arbitrary_bases",
    "title": "Measurements in Arbitrary Orthogonal Bases"
})

It is possible to measure a qubit in orthogonal bases other than the Pauli bases. Suppose one wants to measure a qubit in an orthonormal basis $\ket {b_0}$ and $\ket {b_1}$. Let the state of the qubit be represented by the normalized vector $\ket \psi$. Then, one can always express the state in terms of the basis vectors $\ket{b_0}$ and $\ket{b_1}$, i.e., there exist complex numbers $c_0, c_1$, such that
$$
\ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1}.
$$
The rule for obtaining the probabilities of measurement outcomes is exactly the same as that for the computation basis measurement. For a measurement in a $\{ b_0, b_1\}$ basis we get

- Outcome $b_0$ with probability $|c_0|^2$ and the post-measurement qubit state of $\ket {b_0}$
- Outcome $b_1$ with probability $|c_1|^2$ and the post-measurement qubit state of $\ket {b_1}$

This can be summarized in the following table:
<table>
    <tr>
        <th>Measurement outcome</th>
        <th>Probability of outcome</th>
        <th>State after measurement</th>
    </tr>
    <tr>
        <td>$b_0$</td>
        <td>$|c_0|^2$</td>
        <td>$\ket{b_0}$</td>
    </tr>
    <tr>
        <td>$b_1$</td>
        <td>$|c_1|^2$</td>
        <td>$\ket{b_1}$</td>
    </tr>
</table>

As before, the assumption of $\ket \psi$ being normalized is important, since it guarantees that the two probabilities add to $1$.

> As you may recall, a global phase is said to be hidden or unobservable.
This is explained by the fact that global phases have no impact on quantum measurements. For example, consider two isolated qubits which are in (normalized) states $\ket \psi$ and $e^{i\theta}\ket \psi$.
If both are measured in an orthogonal basis $\{ \ket{b_0},\ket{b_1}\}$, the probabilities of measuring $b_0$ or $b_1$ are identical in both cases, since $|\bra{b_i}\ket{\psi}|^2 = |\bra{b_i}e^{i\theta}\ket{\psi}|^2  $.
Similarly, for either qubit, if $b_i$ is the measurement outcome, the post-measurement state of the qubit is $\ket{b_i}$ for both qubits. Hence, the measurements are independent of the global phase $\theta$.

## Measurements as Projection Operations

Quantum measurements are modeled by orthogonal projection operators. An orthogonal projection operator is a matrix $P$ which satisfies the following property:
$$
P^2 = P^\dagger = P.
$$
(As usual, the $\dagger$ symbol denotes conjugate transposition.)

Using the ket-bra representation, one can represent a projection matrix in the Dirac notation.
For example, one may construct a projector onto the $\ket{0}$ subspace as:
$$
P = \ket 0 \bra 0 \equiv \begin{bmatrix} 1 & 0 \\\ 0 & 0\end{bmatrix}.
$$

A measurement in an orthogonal basis $\{ \ket{b_0}, \ket{b_1}\}$ is described by a pair of projectors $P_0 = \ket{b_0}\bra{b_0}$ and $P_1 = \ket{b_1}\bra{b_1}$. Since $\ket{b_0}$ and $\ket{b_1}$ are orthogonal, their projectors are also orthogonal, i.e., $P_0 P_1 = P_1 P_0 = 0$. The rules for measurements in this basis can then be summarized as follows:

- Measuring a qubit in a state $\ket \psi$ is done by picking one of these projection operators at random.
- Projection $P_0$ is chosen with probability $|P_0 \ket{\psi}|^2$, and the projector $P_1$ is chosen with probability $|P_1\ket{\psi}|^2.$
- If projector $P_0$ is chosen, the post-measurement state of the qubit is given by
$$
\frac1{|P_0 \ket{\psi}|}P_0 \ket\psi,
$$
and similarly for $P_1$.

Although this formalism looks different from the previous sections, it is in fact equivalent. If $\ket \psi = c_0 \ket{b_0} + c_1 \ket{b_1}$, we have
$$
P_0 \ket \psi = c_0 \ket{b_0}, \text{so that } | P_0\ket \psi| = c_0,
$$
and similarly,
$$
P_1 \ket \psi = c_1 \ket{b_1}, \text{so that } |P_1\ket \psi| = c_1.
$$

Thus, as before, the probability of measuring $b_0$ is $|P_0\ket\psi|^2 = |c_0|^2$, and the probability of measuring $b_1$ is $|P_1\ket\psi|^2 = |c_1|^2$. Similarly, one can verify that the post-measurement outcomes are also $\ket{b_0}$ and $\ket{b_1}$ respectively (up to unobservable global phases).

Although the projector formalism for single-qubit systems may seem superfluous, its importance will become clear later while considering measurements for multi-qubit systems.

## Arbitrary Basis Measurements Implementation

In the previous section, we discussed measurements in Pauli bases using the built-in `Measure` operation. We will now show that it is always possible to measure a qubit in any orthogonal basis using just unitary rotation matrices and computation basis measurements.

Consider a state $ \ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1} $ which we would like to measure in an orthonormal basis $\{ \ket{b_0}, \ket{b_1}\}$. First, we construct the following unitary matrix:
$$
U = \ket{0} \bra{b_0} + \ket{1} \bra{b_1}
$$

The conjugate transpose of this unitary is the operator
$$
U^\dagger = \ket{b_0} \bra{0} + \ket{b_1} \bra{1}
$$

(One may verify that $U$ is indeed a unitary matrix, by checking that $U^\dagger U = U U^\dagger = I$)

Note that the effect of these matrices on the two bases is the following:
$$U\ket{b_0} = \ket{0},$$
$$U\ket{b_1} = \ket{1},$$
$$U^\dagger \ket{0} = \ket{b_0},$$
$$U^\dagger \ket 1 = \ket{b_1}.$$

In order to implement a measurement in the ${ \ket{b_0}, \ket{b_1} }$ basis, we do the following:

1. Apply $U$ to $\ket \psi$.  
   The resulting state is $U\ket \psi = c_0 \ket 0 + c_1 \ket 1 $.
2. Measure the state $U\ket{\psi}$ in the computational basis.  
   The outcomes $0$ and $1$ occur with probabilities $|c_0|^2$ and $|c_1|^2$.
3. Apply $U^\dagger$ to the post-measurement state.  
   This transforms the states $\ket 0$ and $\ket 1$ to the states $\ket{b_0}$ and $\ket{b_1}$, respectively.

Thus, $b_0$ and $b_1$ are measured with probabilities $|c_0|^2$ and $|c_1|^2$, respectively, with the end state being $\ket{b_0}$ and $\ket{b_1}$ - which is exactly the measurement we want to implement.

This procedure can be used to distinguish arbitrary orthogonal states as well, as will become clear from the following exercises.

## 🔎 Analyze

**The outcome probabilities for a measurement in a specified basis**

1. What are the outcome probabilities of measuring a qubit in the $\ket{0}$ state in the Pauli X basis, i.e., the $\{ \ket +, \ket -\}$ basis?

2. What are the outcome probabilities of measuring a qubit in the $0.6\ket{0} + 0.8 \ket{1}$ state in the Pauli Y basis, i.e., the $\{ \ket i, \ket{-i}\}$ basis?

<details>
<summary><b>Solution</b></summary>

1. To find the probabilities of measuring $+$ and $-$, we first need to express the state $\ket 0$ in terms of $\ket +$ and $\ket -$. Using the fact that $\ket{\pm} = \frac{1}{\sqrt{2}}  (\ket{0} \pm \ket{1})$, we can show that
    $$
    \ket 0 = \frac{1}{\sqrt{2}} \ket{+} + \frac{1}{\sqrt{2}} \ket{-}.
    $$
    Thus, the probability of measuring $+$ is $|\frac1{\sqrt2}|^2 = 0.5$, and similarly, the probability of measuring $-$ is $0.5$.

2. Similar to the first part, we need to express the state $\ket \psi = 0.6 \ket 0 + 0.8 \ket 1$ in the $\ket{\pm i}$ basis. For this calculation, we use the projection matrix approach.

    First, we recall that the states $\ket{\pm i}$ are given by
    $$
    \ket{\pm i} = \frac1{\sqrt2} (\ket 0 \pm i \ket 1).
    $$

    We can now construct the two projectors $P_{\pm i}$ onto states $\ket {\pm i}$ as follows:
    $$P_{i} = \ket{i}\bra{i} = \frac{1}{2} \begin{bmatrix} 1 \\\\ i \end{bmatrix} \begin{bmatrix} 1 & -i \end{bmatrix} = \frac{1}{2} \begin{bmatrix}1 & -i \\\\ i & 1\end{bmatrix},$$
    $$P_{-i} = \ket{-i}\bra{-i} = \frac{1}{2} \begin{bmatrix} 1 \\\\ -i \end{bmatrix} \begin{bmatrix} 1 & i \end{bmatrix} = \frac{1}{2} \begin{bmatrix}1 & i \\\\ -i & 1\end{bmatrix}.$$

    Recalling that the probabilities of measuring $\pm i$ are equal to the norm of the vectors $P_{\pm i}\ket \psi$, we now apply $P_{\pm i}$ to $\ket \psi$:
    $$P_{+i} \ket \psi = \frac{1}{2} \begin{bmatrix}1 & -i \\\\ i & 1\end{bmatrix} \begin{bmatrix} 0.6 \\\\ 0.8 \end{bmatrix} = \frac{1}{2} \begin{bmatrix} 0.6 - 0.8i \\\\ 0.8 + 0.6i \end{bmatrix},$$
    $$P_{-i} \ket \psi = \frac{1}{2} \begin{bmatrix}1 & i \\\\ -i & 1\end{bmatrix} \begin{bmatrix} 0.6 \\\\ 0.8 \end{bmatrix} = \frac{1}{2} \begin{bmatrix} 0.6 + 0.8i \\\\ 0.8 - 0.6i \end{bmatrix}.$$

    Hence, the probabilities of measuring $\pm i$, which we denote by $p(\pm i)$, are:
    $$p(+i) = |P_{+i} \ket \psi|^2 = \frac{1}{4}(|0.6 - 0.8i|^2 + |0.8 + 0.6i|^2) = \frac{1}{2},$$
    $$p(-i) = |P_{-i} \ket \psi|^2 = \frac{1}{4}(|0.6 + 0.8i|^2 + |0.8 - 0.6i|^2) = \frac{1}{2}.$$

</details>

@[exercise]({
    "id": "distinguish_orthogonal_states_1",
    "title": "Distinguishing Orthogonal States: 1",
    "descriptionPath": "./distinguish_orthogonal_states_1/index.md",
    "placeholderSourcePath": "./distinguish_orthogonal_states_1/placeholder.qs",
    "solutionPath": "./distinguish_orthogonal_states_1/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./distinguish_orthogonal_states_1/verification.qs"
    ]
})

@[exercise]({
    "id": "distinguish_orthogonal_states_2",
    "title": "Distinguishing Orthogonal States: 2",
    "descriptionPath": "./distinguish_orthogonal_states_2/index.md",
    "placeholderSourcePath": "./distinguish_orthogonal_states_2/placeholder.qs",
    "solutionPath": "./distinguish_orthogonal_states_2/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./distinguish_orthogonal_states_2/verification.qs"
    ]
})

@[exercise]({
    "id": "a_b_basis_measurements",
    "title": "Measurement In The |A〉, |B〉 Basis",
    "descriptionPath": "./a_b_basis_measurements/index.md",
    "placeholderSourcePath": "./a_b_basis_measurements/placeholder.qs",
    "solutionPath": "./a_b_basis_measurements/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./a_b_basis_measurements/verification.qs"
    ]
})

@[section]({
    "id": "single_qubit_measurements_conclusion",
    "title": "Conclusion"
})

Congratulations!
You can continue to the next Kata in the series to learn about measurements in multi-qubit systems.
