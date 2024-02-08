# T operation

`operation T(qubit : Qubit) : Unit is Adj + Ctl`

## Summary
Applies the π/8 gate to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    T \mathrel{:=}
    \begin{bmatrix}
        1 & 0 \\\\
        0 & e^{i \pi / 4}
    \end{bmatrix}.
\end{align}
$$
