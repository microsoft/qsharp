# CCNOT operation

`operation CCNOT(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl`

## Summary
Applies the doubly controlled–NOT (CCNOT) gate to three qubits.

## Input
### control1
First control qubit for the CCNOT gate.
### control2
Second control qubit for the CCNOT gate.
### target
Target qubit for the CCNOT gate.

## Remarks
Equivalent to:
```qsharp
Controlled X([control1, control2], target);
```
