# Measurements in systems with multiple qubits

In the previous tutorial, we discussed the concept of measurements done on single-qubit systems.
Building upon those ideas, this tutorial will introduce you to measurements done on multi-qubit systems, and how to implement such measurements in Q#.
This will include measuring a single qubit in a multi-qubit system, as well as measuring multiple qubits simultaneously.

We recommend to go through the [tutorial that introduces single-qubit system measurements](../SingleQubitSystemMeasurements/SingleQubitSystemMeasurements.ipynb) before starting this one.
$\renewcommand{\ket}[1]{\left\lvert#1\right\rangle}$
$\renewcommand{\bra}[1]{\left\langle#1\right\rvert}$

You should be familiar with the following concepts before tackling the Single-Qubit System Measurements tutorial (and this workbook):

1. Basic linear algebra
2. Single and multi-qubit systems
3. Single and multi-qubit gates
   $\renewcommand{\ket}[1]{\left\lvert#1\right\rangle}$
   $\renewcommand{\bra}[1]{\left\langle#1\right\rvert}$

## Types of measurements on multi-qubit systems

There are several types of measurements you can perform on an $n$-qubit system ($n>1$):

- Measuring all the qubits simultaneously in an orthogonal basis ($2^n$ possible outcomes). As we shall see below, this is a direct generalization of orthogonal basis measurements done in single-qubit systems introduced in the previous tutorial.
- Partial measurement: measuring $m$ qubits out of $n$, for $m<n$ ($2^m$ possible outcomes). Partial measurements involve a partial collapse of the system's wave function, since only some of the qubits are measured.
- Joint measurement: measuring a joint property of all $n$ qubits ($2$ possible outcomes).

We will discuss these concepts in the same order as in the list above.

## Full measurements: measurements in multi-qubit bases

Consider a system consisting of $n\geq1$ qubits. The wave function of such a system belongs to a vector space of dimension $2^n$. Thus, the vector space is spanned by an orthogonal basis, such as the computational basis which consists of the vectors $|0\dotsc0\rangle, \dotsc, |1\dotsc 1\rangle$. For generality, we consider an arbitrary orthonormal basis, which we denote by $\{ |b_0\rangle, |b_1\rangle, \dotsc, |b_{2^n-1}\rangle \}$.

Then, the state $|\psi\rangle$ of the multi-qubit system can be expressed as a linear combination of the $2^n$ basis vectors $|b_i\rangle$. That is, there exist complex numbers $c_0,c_1,\dotsc, c_{2^n-1}$ such that

$$
|\psi\rangle = \sum_{i=0}^{2^n-1} c_i|b_i\rangle \equiv \begin{pmatrix}c_0 \\ c_1 \\ \vdots \\ c_{2^n-1}\end{pmatrix}.
$$

In line with the usual convention, we choose the wave function to be normalized, so that $|c_0|^2 + \dotsc + |c_{2^n-1}|^2 =1$. Then, a quantum measurement in the $\{ |b_0\rangle, |b_1\rangle, \dotsc, |b_{2^n-1}\rangle \}$ basis satisfies the following rules:

- The measurement outcome $b_i$ occurs with probability $|c_i|^2$.
- Whenever the measurement outcome is $b_i$, the wave function collapses to the state $|b_i\rangle$. That is, the post-measurement state of the system is equal to $|b_i\rangle$.

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
        <td style="text-align:center; border:1px solid">$b_i$</td>
        <td style="text-align:center; border:1px solid">$|c_i|^2$</td>
        <td style="text-align:center; border:1px solid">$\ket{b_i}$</td>
    </tr>    
</table>
 
 
> Similar to measurements in single-qubit systems, the assumption of normalization of the original wave function is required in order to ensure that the sum of all the outcome probabilities is 1.
