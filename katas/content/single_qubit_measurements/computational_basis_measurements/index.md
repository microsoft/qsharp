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
