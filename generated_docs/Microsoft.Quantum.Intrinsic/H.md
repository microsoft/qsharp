# H operation

`operation H(qubit : Qubit) : Unit is Adj + Ctl`

## Summary
Applies the Hadamard transformation to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    e^{i \theta [P_0 \otimes P_1 \cdots P_{N-1}]},
\end{align}
$$
where $P_i$ is the $i$th element of `paulis`, and where
$N = $`Length(paulis)`.
