# operation ApplyP(pauli : Pauli, target : Qubit) : Unit is Adj + Ctl

## Summary
Given a single-qubit Pauli operator, applies the corresponding operation
to a single qubit.

## Input
### pauli
The Pauli operator to be applied.
### target
The qubit to which `pauli` is to be applied as an operation.

## Example
The following are equivalent:
```qsharp
ApplyP(PauliX, q);
```
and
```qsharp
X(q);
```
