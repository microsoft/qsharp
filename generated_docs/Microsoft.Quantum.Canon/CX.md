# operation CX(control : Qubit, target : Qubit) : Unit is Adj + Ctl

## Summary
Applies the controlled-X (CX) gate to a pair of qubits.

## Input
### control
Control qubit for the CX gate.
### target
Target qubit for the CX gate.

## Remarks
This operation can be simulated by the unitary matrix
$$
\begin{align}
    \left(\begin{matrix}
        1 & 0 & 0 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1 \\\\
        0 & 0 & 1 & 0
     \end{matrix}\right)
\end{align},
$$
where rows and columns are organized as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled X([control], target);
```
and to:
```qsharp
CNOT(control, target);
```
