## Joint measurements

Joint measurements, also known as Pauli measurements, are a generalization of 2-outcome measurements to multiple qubits and other bases. In Q#, joint measurements in Pauli bases are implemented using the [Measure](https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure) operation. Let's review single-qubit measurements in a different light before discussing joint measurements. 

### Single-qubit Pauli measurement
For single-qubit systems, any measurement corresponding to an orthogonal basis can be associated with a Hermitian matrix with eigenvalues $\pm 1$. The possible measurement outcomes (represented as `Result` in Q#) are the eigenvalues of the Hermitian matrix, and the corresponding projection matrices for the measurement are the projection operators onto the *eigenspaces* corresponding to the eigenvalues. 

For example, consider the computational basis measurement, which can result in outcomes `Zero` or `One` corresponding to states $\ket 0$ and $\ket 1$. This measurement is associated with the Pauli Z operator, which is given by 
$$
Z = \begin{pmatrix} 1 & 0 \\ 0 & -1\end{pmatrix} = \ket{0}\bra{0} - \ket{1}\bra{1}.
$$
The $Z$ operator has two eigenvalues, $1$ and $-1$, with corresponding eigenvectors $\ket{0}$ and $\ket{1}$. A $Z$-measurement is then a measurement in the $\{\ket{0},\ket{1}\}$ basis, with the measurement outcomes being $1$ and $-1$ respectively. In Q#, by convention, an eigenvalue of $1$ corresponds to a `Result` of `Zero`, while an eigenvalue of $-1$ corresponds to a `Result` of `One`.

Similarly, one can implement measurements corresponding to the Pauli X and Y operators. We summarize the various properties below:
<table style="border:1px solid">
    <col width=200>
    <col width=50>
    <col width=100>
    <col width=150>
    <col width=150>
    <tr>
        <th style="text-align:center; border:1px solid">Pauli Operator</th>
        <th style="text-align:center; border:1px solid">Matrix</th>
        <th style="text-align:center; border:1px solid">Eigenvalue</th>
        <th style="text-align:center; border:1px solid">Eigenvector/post-measurement state</th>
        <th style="text-align:center; border:1px solid">Measurement Result in Q#</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid" rowspan="2">$X$</td>
        <td style="text-align:center; border:1px solid" rowspan="2">$\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$</td>
        <td style="text-align:center; border:1px solid">+1</td>
        <td style="text-align:center; border:1px solid">$\ket{+}$</td>
        <td style="text-align:center; border:1px solid">Zero</td>
    </tr><tr>
        <td style="text-align:center; border:1px solid">-1</td>
        <td style="text-align:center; border:1px solid">$\ket{-}$</td>
        <td style="text-align:center; border:1px solid">One</td>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid" rowspan="2">$Y$</td>
        <td style="text-align:center; border:1px solid" rowspan="2">$\begin{bmatrix} 0 & -i \\ i & 0 \end{bmatrix}$</td>
        <td style="text-align:center; border:1px solid">+1</td>
        <td style="text-align:center; border:1px solid">$\ket{i}$</td>
        <td style="text-align:center; border:1px solid">Zero</td>
    </tr><tr>
        <td style="text-align:center; border:1px solid">-1</td>
        <td style="text-align:center; border:1px solid">$\ket{-i}$</td>
        <td style="text-align:center; border:1px solid">One</td>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid" rowspan="2">$Z$</td>
        <td style="text-align:center; border:1px solid" rowspan="2">$\begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$</td>
        <td style="text-align:center; border:1px solid">+1</td>
        <td style="text-align:center; border:1px solid">$\ket{0}$</td>
        <td style="text-align:center; border:1px solid">Zero</td>
    </tr><tr>
        <td style="text-align:center; border:1px solid">-1</td>
        <td style="text-align:center; border:1px solid">$\ket{1}$</td>
        <td style="text-align:center; border:1px solid">One</td>
    </tr>
</table>

In general, any measurement on a single qubit which results in two outcomes corresponds to the Hermitian operator $U Z U^\dagger$, for some $2\times 2$ unitary matrix $U$.

Joint measurements are a generalization of this principle for multi-qubit matrices.


### Parity measurements
The simplest joint measurement is a parity measurement. A parity measurement treats computational basis vectors differently depending on whether the number of 1's in the basis vector is even or odd. 

For example, the operator $Z\otimes Z$, or $ZZ$ in short, is the parity measurement operator for a two-qubit system. The eigenvalues $1$ and $-1$ correspond to the subspaces spanned by basis vectors $\{ |00\rangle, |11\rangle \}$ and $\{ |01\rangle, |10\rangle \}$, respectively. That is, when a $ZZ$ measurement results in a `Zero` (i.e. the eigenvalue $+1$), the post-measurement state is a superposition of only those computational basis vectors which have an even number of $1$'s. On the other hand, a result of `One` corresponds to a post-measurement state with only odd parity computational basis vectors.

> Let's see what happens to various two-qubit states after the parity measurement. The $Z \otimes Z$ matrix for two qubits is: 
>
>$$Z \otimes Z = \begin{bmatrix}
    1 & 0 & 0 & 0 \\
    0 & -1 & 0 & 0 \\
    0 & 0 & -1 & 0 \\
    0 & 0 & 0 & 1 \\
\end{bmatrix}$$
>
>When this transformation is applied to a basis state $|00\rangle$, we get
>
> $$\begin{bmatrix}
    1 & 0 & 0 & 0 \\
    0 & -1 & 0 & 0 \\
    0 & 0 & -1 & 0 \\
    0 & 0 & 0 & 1 \\
\end{bmatrix} 
\begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \\ \end{bmatrix} = 
\begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \\ \end{bmatrix}$$
>
> Comparing this to the characteristic equation for eigenvectors of $Z \otimes Z$ given by
$ Z \otimes Z |\psi\rangle = \lambda |\psi\rangle$,
it is easy to see that $|00\rangle$ belongs to the $+1$ eigenspace, hence the $Z \otimes Z$ measurement will return `Zero` and leave the state unchanged.
>
> Similarly, it can easily be verified that $|11\rangle$ also belongs to $+1$ eigenspace, while $|01\rangle$ and $|10\rangle$ belong to the $-1$ eigenspace.
> 
> Now, what happens if we apply a $Z \otimes Z$ measurement to a superposition state $\alpha |00\rangle + \beta |11\rangle$? We can see that 
>
> $$\begin{bmatrix}
    1 & 0 & 0 & 0 \\
    0 & -1 & 0 & 0 \\
    0 & 0 & -1 & 0 \\
    0 & 0 & 0 & 1 \\
\end{bmatrix} 
\begin{bmatrix} \alpha \\ 0 \\ 0 \\ \beta \\ \end{bmatrix} = 
\begin{bmatrix} \alpha \\ 0 \\ 0 \\ \beta \\ \end{bmatrix}$$
>
>So this state also belongs to the $+1$ eigenspace, and measuring it will return `Zero` and leave the state unchanged. Similarly, we can verify that an $\alpha |01\rangle + \beta |10\rangle$ state belongs to the $-1$ eigenspace, and measuring it will return `One` without changing the state.

Similarly, a parity measurement on a higher number of qubits can be implemented using a $Z \otimes \dotsc \otimes Z$ measurement.

### <span style="color:blue">Exercise 9</span>: Two-qubit parity measurement

**Inputs**: Two qubits stored in an array which are guaranteed to be either in a superposition of the states $|00\rangle$ and $|11\rangle$ or in a superposition of states $|01\rangle$ and $|10\rangle$.

**Output**: 0 if qubits were in the first superposition, 1 if they were in the second superposition.  
*The state of the qubits at the end of the operation should be the same as the starting state.*
