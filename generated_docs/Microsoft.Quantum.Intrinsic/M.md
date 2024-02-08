# M operation

`operation M(qubit : Qubit) : Result`

## Summary
Performs a measurement of a single qubit in the
Pauli _Z_ basis.

## Input
### qubit
Qubit to be measured.

## Output
`Zero` if the +1 eigenvalue is observed, and `One` if
the -1 eigenvalue is observed.

## Remarks
The output result is given by
the distribution
$$
\begin{align}
    \Pr(\texttt{Zero} | \ket{\psi}) =
        \braket{\psi | 0} \braket{0 | \psi}.
\end{align}
$$

Equivalent to:
```qsharp
Measure([PauliZ], [qubit]);
```
