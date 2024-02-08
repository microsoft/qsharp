# operation CNOT(control : Qubit, target : Qubit) : Unit is Adj + Ctl

## Summary
Applies the controlled-NOT (CNOT) gate to a pair of qubits.

## Input
### control
Control qubit for the CNOT gate.
### target
Target qubit for the CNOT gate.

## Remarks
$$
\begin{align}
    \operatorname{CNOT} \mathrel{:=}
    \begin{bmatrix}
        1 & 0 & 0 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1 \\\\
        0 & 0 & 1 & 0
    \end{bmatrix},
\end{align}
$$

where rows and columns are ordered as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled X([control], target);
```
