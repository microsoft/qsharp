# CZ operation

`operation CZ(control : Qubit, target : Qubit) : Unit is Adj + Ctl`

## Summary
Applies the controlled-Z (CZ) gate to a pair of qubits.

## Input
### control
Control qubit for the CZ gate.
### target
Target qubit for the CZ gate.

## Remarks
This operation can be simulated by the unitary matrix
$$
\begin{align}
    1 & 0 & 0 & 0 \\\\
    0 & 1 & 0 & 0 \\\\
    0 & 0 & 1 & 0 \\\\
    0 & 0 & 0 & -1
\end{align},
$$
where rows and columns are organized as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled Z([control], target);
```
