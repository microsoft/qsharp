# Measurements for single-qubit systems

This tutorial introduces you to measurements done on single-qubit systems.
The concept of a measurement is a central part of quantum mechanics,
as well as quantum algorithms. Single-qubit measurements, as their name implies
are measurements on single qubits. The outcomes of a measurement in quantum mechanics
are probabilistic, and in general, change the state of the qubit
depending on the outcome of the measurement.

We recommend to go through the
[tutorial that introduces single-qubit gates](../single_qubit_gates/content.md)
before starting this one.

**This tutorial covers the following topics:**

* Computational basis measurements
* Pauli basis measurements
* Measurements in arbitrary orthogonal bases
* Representing measurements as projector operators
  $\renewcommand{\ket}[1]{\left\lvert#1\right\rangle}$
  $\renewcommand{\bra}[1]{\left\langle#1\right\rvert}$

**What you should know for this workbook**

You should be familiar with the following concepts before tackling the Single-Qubit System Measurements tutorial:

1. Basic linear algebra
1. The concept of a qubit
1. Single-qubit gates
$\renewcommand{\ket}[1]{\left\lvert#1\right\rangle}$
$\renewcommand{\bra}[1]{\left\langle#1\right\rvert}$

# Computational basis measurements

In this section, we will discuss the simplest type of qubit measurements -
measurements in the computational basis. (This is the "default" type of measurements -
unless otherwise specified, "measurement" refers to this type.)

The state $\ket{\psi}$ of a single qubit can always be expressed in
[Dirac notation](../../qubit/content.md##Dirac-Notation) as
$$\ket{\psi} = \alpha \ket{0} + \beta \ket{1},$$

where $\alpha$ and $\beta$ are complex numbers, and the state is normalized,
i.e., $|\alpha|^2 + |\beta|^2 = 1$.

We can examine the qubit to get some information about its state -
*measure* its state. Similar to the classical case of examining a bit,
the outcome of a measurement can be $0$ or $1$. But, unlike the classical case,
quantum measurement is a probabilistic process.
The probabilities of the measurement outcomes being $0$ and $1$ are
$|\alpha|^2$ and $|\beta|^2$, respectively. Additionally, the state
of the qubit is modified by the measurement: if the outcome of the measurement
is $0$, then the post-measurement state of the qubit is $\ket{0}$,
and if the outcome is $1$, the state is $\ket{1}$. In quantum mechanics,
this is referred to as the [collapse of the wave function](https://en.wikipedia.org/wiki/Wave_function_collapse).

Computational basis measurement outcomes and their probabilities are summarized in the table below:
<table style="border:1px solid">
    <col width=150>
    <col width=150>
    <col width=150>
    <tr>
        <th style=\"text-align:center; border:1px solid\">Measurement outcome</th>
        <th style=\"text-align:center; border:1px solid\">Probability of outcome</th>
        <th style=\"text-align:center; border:1px solid\">State after measurement</th>
    </tr>
    <tr>
        <td style=\"text-align:center; border:1px solid\">$0$</td>
        <td style=\"text-align:center; border:1px solid\">$|\alpha|^2$</td>
        <td style=\"text-align:center; border:1px solid\">$\ket 0$</td>
    </tr>
    <tr>
        <td style=\"text-align:center; border:1px solid\">$1$</td>
        <td style=\"text-align:center; border:1px solid\">$|\beta|^2$</td>
        <td style=\"text-align:center; border:1px solid\">$\ket 1$</td>
    </tr>
</table>

>Unlike quantum gates which are unitary and reversible operations,
>measurements are neither unitary nor reversible. Since the outcomes
>of a measurement are probabilistic, any two isolated qubits which are
>initially prepared in identical superposition states are in general
>not guaranteed to have the same measurement outcomes after each qubit
>has been measured separately. As we will see below, measurements are
>modeled by projection operators instead of unitary operators.
>
>Additionally, the assumption of the wave function being **normalized**
>is important, since the probability outcomes must sum up to $1$.
>If the wave function is not normalized, it is important to normalize it first
>in order to obtain the correct measurement probabilities.

@[question]({
"id": "probabilities_specific_state",
"descriptionPath": "./probabilities_specific_state/index.md",
"answerPath": "./probabilities_specific_state/solution.md"
})

## <span style="color:blue">Demo: Implementing measurement in Q# using the M operation</span>

In this demo, we prepare a qubit in the state we've seen in Exercise 1, and then measure it in the computational basis. In Q#, single-qubit measurements in the computational basis can be implemented using the [M operation](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.m). It will return the constant `Zero` if measurement result was $0$ or the constant `One` if the measurement result was $1$. `Zero` and `One` are constants of type `Result`.

> If you run this code multiple times, you will notice that whenever the measurement outcome is $1$, the post-measurement state of the qubit is $\ket 1$, and similarly for $0$. This is in line with our expectation that after the measurement the wave function 'collapses' to the corresponding state.

@[example]({
"id": "implementing_measurement",
"codePath": "./implementing_measurement/example.qs"
})

## <span style="color:blue">Demo: Measurement statistics</span>

The following cell contains code demonstrating that the theoretical and experimental values of the probability outcomes indeed match with each other. We repeatedly prepare the same state $\ket \psi = 0.6 \ket 0 + 0.8 \ket 1$ and measure it in the computational basis $100$ times. At the end, we expect $0$ to be measured approximately $36$ times, and $1$ to be measured approximately $64$ times. Note that since measurements are probabilistic, we do not expect the results to match these values exactly.

@[example]({
"id": "measurement_statistics",
"codePath": "./measurement_statistics/example.qs"
})

Measurements can be used to distinguish orthogonal states. We start with an exercise for distinguishing between the computational basis states, and discuss the general case of arbitrary basis measurements later in the tutorial.

@[exercise]({
"id": "distinguish_0_and_1",
"descriptionPath": "./distinguish_0_and_1/index.md",
"placeholderSourcePath": "./distinguish_0_and_1/placeholder.qs",
"verificationSourcePath": "./distinguish_0_and_1/verify.qs",
"solutionPath": "./distinguish_0_and_1/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

## Measurements in the Pauli bases
So far, we have discussed measurements done in the computational basis, i.e., the $\{ \ket 0, \ket 1\}$ basis. 

It is also possible to implement measurements in other orthogonal bases, such as the [Pauli X basis](../SingleQubitGates/SingleQubitGates.ipynb#Pauli-Gates), which consists of the two vectors $\ket + = \frac1{\sqrt2} \big(\ket 0 +\ket 1\big)$, and $\ket - = \frac1{\sqrt2} \big(\ket 0 -\ket 1\big)$. Q# has a built-in operation [Measure](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure) for measurements in the Pauli bases. 

> The `Measure` operation can be used for measuring multiple qubits in a multi-qubit system; however, in this tutorial we only consider measurements for single-qubit systems.

The eigenvalues of a Pauli matrix are $\pm 1$, with one eigenvector corresponding to each eigenvalue. For any chosen Pauli basis, the `Measure` operation returns `Zero` if the measurement outcome corresponds to the eigenvalue $+1$, and returns `One` if the measurement outcome corresponds to the eigenvalue $-1$. As in the case of the computational basis measurements, the wave function of the qubit collapses to the corresponding state after the measurement is executed. 

The probabilities of the outcomes are defined using a similar rule: to measure a state $\ket \psi$ in a Pauli basis $\{ \ket {b_0}, \ket {b_1}\}$, we represent it as a linear combination of the basis vectors
$$\ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1}$$

The probabilities of outcomes $0$ and $1$ will be defined as $|c_0|^2$ and $|c_1|^2$, respectively.

> Computational basis measurement is often referred to as measurement in Pauli Z basis. Indeed, the eigenvectors of the Z gate are $\ket 0$ and $\ket 1$, with eigenvalues $+1$ and $-1$, respectively.

@[exercise]({
"id": "distinguish_plus_and_minus",
"descriptionPath": "./distinguish_plus_and_minus/index.md",
"placeholderSourcePath": "./distinguish_plus_and_minus/placeholder.qs",
"verificationSourcePath": "./distinguish_plus_and_minus/verification.qs",
"solutionPath": "./distinguish_plus_and_minus/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

## Measurements in arbitrary orthogonal bases
It is possible to measure a qubit in orthogonal bases other than the Pauli bases. Suppose one wants to measure a qubit in an orthonormal basis $\ket {b_0}$ and $\ket {b_1}$. Let the state of the qubit be represented by the normalized vector $\ket \psi$. Then, one can always express the state in terms of the basis vectors $\ket{b_0}$ and $\ket{b_1}$, i.e., there exist complex numbers $c_0, c_1$, such that 
$$
\ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1}.
$$
The rule for obtaining the probabilities of measurement outcomes is exactly the same as that for the computation basis measurement. For a measurement in a $\{ b_0, b_1\}$ basis we get
- Outcome $b_0$ with probability $|c_0|^2$, and the post-measurement state of the qubit $\ket {b_0}$;
- Outcome $b_1$ with probability $|c_1|^2$, and the post-measurement state of the qubit $\ket {b_1}$.

This can be summarized in the following table:
<table style="border:1px solid">
    <col width=150>
    <col width=150>
    <col width=150>
    <tr>
        <th style="text-align:center; border:1px solid">Measurement outcome</th>
        <th style="text-align:center; border:1px solid">Probability of outcome</th>
        <th style="text-align:center; border:1px solid">State after measurement</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$b_0$</td>
        <td style="text-align:center; border:1px solid">$|c_0|^2$</td>
        <td style="text-align:center; border:1px solid">$\ket{b_0}$</td>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$b_1$</td>
        <td style="text-align:center; border:1px solid">$|c_1|^2$</td>
        <td style="text-align:center; border:1px solid">$\ket{b_1}$</td>
    </tr>
    
</table>

As before, the assumption of $\ket \psi$ being normalized is important, since it guarantees that the two probabilities add to $1$.

> As you may recall, a [global phase](../Qubit/Qubit.ipynb#Relative-and-Global-Phase) is said to be hidden or unobservable. 
This is explained by the fact that global phases have no impact on quantum measurements. For example, consider two isolated qubits which are in (normalized) states $\ket \psi$ and $e^{i\theta}\ket \psi$. 
If both are measured in an orthogonal basis $\{ \ket{b_0},\ket{b_1}\}$, the probabilities of measuring $b_0$ or $b_1$ are identical in both cases, since $|\bra{b_i}\ket{\psi}|^2 = |\bra{b_i}e^{i\theta}\ket{\psi}|^2  $. 
Similarly, for either qubit, if $b_i$ is the measurement outcome, the post-measurement state of the qubit is $\ket{b_i}$ for both qubits. Hence, the measurements are independent of the global phase $\theta$.

> ### Measurements as projection operations
Quantum measurements are modeled by orthogonal projection operators. An orthogonal projection operator is a matrix $P$ which satisfies the following property:
$$
P^2 = P^\dagger = P.
$$
(As usual, the $\dagger$ symbol denotes conjugate transposition.) 
>
> Using the [ket-bra representation](../SingleQubitGates/SingleQubitGates.ipynb#Ket-bra-Representation), one can represent a projection matrix in the Dirac notation.
For example, one may construct a projector onto the $\ket{0}$ subspace as:
$$
P = \ket 0 \bra 0 \equiv \begin{bmatrix} 1 & 0 \\ 0 & 0\end{bmatrix}.
$$
>
>A measurement in an orthogonal basis $\{ \ket{b_0}, \ket{b_1}\}$ is described by a pair of projectors $P_0 = \ket{b_0}\bra{b_0}$ and $P_1 = \ket{b_1}\bra{b_1}$. Since $\ket{b_0}$ and $\ket{b_1}$ are orthogonal, their projectors are also orthogonal, i.e., $P_0 P_1 = P_1 P_0 = 0$. The rules for measurements in this basis can then be summarized as follows: 
- Measuring a qubit in a state $\ket \psi$ is done by picking one of these projection operators at random.
- Projection $P_0$ is chosen with probability $|P_0 \ket{\psi}|^2$, and the projector $P_1$ is chosen with probability $|P_1\ket{\psi}|^2$.
- If projector $P_0$ is chosen, the post-measurement state of the qubit is given by
$$
\frac1{|P_0 \ket{\psi}|}P_0 \ket\psi,
$$
and similarly for $P_1$.
>
>Although this formalism looks different from the previous sections, it is in fact equivalent. If $\ket \psi = c_0 \ket{b_0} + c_1 \ket{b_1}$, we have 
$$
P_0 \ket \psi = c_0 \ket{b_0}, \text{so that } | P_0\ket \psi| = c_0,
$$
and similarly, 
$$
P_1 \ket \psi = c_1 \ket{b_1}, \text{so that } |P_1\ket \psi| = c_1.
$$
>
>Thus, as before, the probability of measuring $b_0$ is $|P_0\ket\psi|^2 = |c_0|^2$, and the probability of measuring $b_1$ is $|P_1\ket\psi|^2 = |c_1|^2$. Similarly, one can verify that the post-measurement outcomes are also $\ket{b_0}$ and $\ket{b_1}$ respectively (up to unobservable global phases).
>
>Although the projector formalism for single-qubit systems may seem superfluous, its importance will become clear later, while considering measurements for multi-qubit systems.

### Arbitrary basis measurements implementation
In the previous section, we discussed measurements in Pauli bases using the built-in `Measure` operation. We will now show that using just unitary rotation matrices and computation basis measurements it is always possible to measure a qubit in any orthogonal basis. 

Consider a state $ \ket \psi = c_0 \ket {b_0} + c_1 \ket {b_1} $ which we would like to measure in an orthonormal basis $\{ \ket{b_0}, \ket{b_1}\}$. First, we construct the following unitary matrix:
$$
U = \ket{0} \bra{b_0} + \ket{1} \bra{b_1}
$$

The conjugate transpose of this unitary is the operator 
$$
U^\dagger = \ket{b_0} \bra{0} + \ket{b_1} \bra{1}
$$

(One may verify that $U$ is indeed a unitary matrix, by checking that $U^\dagger U = U U^\dagger = I$.)

Note that the effect of these matrices on the two bases is the following:
\begin{align}
U\ket{b_0} &= \ket{0}; & U\ket{b_1} &= \ket{1}\\
U^\dagger \ket{0} &= \ket{b_0}; & U^\dagger \ket 1 &= \ket{b_1}.
\end{align}

In order to implement a measurement in the $\{ \ket{b_0}, \ket{b_1}\}$ basis, we do the following:

1. Apply $U$ to $\ket \psi$.  
   The resulting state is $U\ket \psi = c_0 \ket 0 + c_1 \ket 1 $.
2. Measure the state $U\ket{\psi}$ in the computational basis.  
   The outcomes $0$ and $1$ occur with probabilities $|c_0|^2$ and $|c_1|^2$.
3. Apply $U^\dagger$ to the post-measurement state.  
   This transforms the states $\ket 0$ and $\ket 1$ to the states $\ket{b_0}$ and $\ket{b_1}$, respectively.

Thus, $b_0$ and $b_1$ are measured with probabilities $|c_0|^2$ and $|c_1|^2$, respectively, with the end state being $\ket{b_0}$ and $\ket{b_1}$ - which is exactly the measurement we want to implement. 

This procedure can be used to distinguish arbitrary orthogonal states as well, as will become clear from the following exercises.

@[question]({
"id": "probabilities_specific_basis",
"descriptionPath": "./probabilities_specific_basis/index.md",
"answerPath": "./probabilities_specific_basis/solution.md"
})

@[exercise]({
"id": "distinguish_orthogonal_states_1",
"descriptionPath": "./distinguish_orthogonal_states_1/index.md",
"placeholderSourcePath": "./distinguish_orthogonal_states_1/placeholder.qs",
"verificationSourcePath": "./distinguish_orthogonal_states_1/verification.qs",
"solutionPath": "./distinguish_orthogonal_states_1/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "distinguish_orthogonal_states_2",
"descriptionPath": "./distinguish_orthogonal_states_2/index.md",
"placeholderSourcePath": "./distinguish_orthogonal_states_2/placeholder.qs",
"verificationSourcePath": "./distinguish_orthogonal_states_2/verification.qs",
"solutionPath": "./distinguish_orthogonal_states_2/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

@[exercise]({
"id": "a_b_basis_measurements",
"descriptionPath": "./a_b_basis_measurements/index.md",
"placeholderSourcePath": "./a_b_basis_measurements/placeholder.qs",
"verificationSourcePath": "./a_b_basis_measurements/verification.qs",
"solutionPath": "./a_b_basis_measurements/solution.md",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./common.qs"
]
})

## Conclusion

Congratulations! You have learned enough to try solving the first part of the [Measurements kata](../../Measurements/Measurements.ipynb). 
When you are done with that, you can continue to the next tutorial in the series to learn about [measurements for multi-qubit systems](../MultiQubitSystemMeasurements/MultiQubitSystemMeasurements.ipynb).
