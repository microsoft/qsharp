# operation Measure(bases : Pauli[], qubits : Qubit[]) : Result

## Summary
Performs a joint measurement of one or more qubits in the
specified Pauli bases.

## Input
### bases
Array of single-qubit Pauli values indicating the tensor product
factors on each qubit.
### qubits
Register of qubits to be measured.

## Output
`Zero` if the +1 eigenvalue is observed, and `One` if
the -1 eigenvalue is observed.

## Remarks
The output result is given by the distribution:
$$
\begin{align}
    \Pr(\texttt{Zero} | \ket{\psi}) =
        \frac12 \braket{
            \psi \mid|
            \left(
                \boldone + P_0 \otimes P_1 \otimes \cdots \otimes P_{N-1}
            \right) \mid|
            \psi
        },
\end{align}
$$
where $P_i$ is the $i$th element of `bases`, and where
$N = \texttt{Length}(\texttt{bases})$.
That is, measurement returns a `Result` $d$ such that the eigenvalue of the
observed measurement effect is $(-1)^d$.

If the basis array and qubit array are different lengths, then the
operation will fail.
