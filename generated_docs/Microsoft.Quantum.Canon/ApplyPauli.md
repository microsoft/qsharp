# operation ApplyPauli(pauli : Pauli[], target : Qubit[]) : Unit is Adj + Ctl

## Summary
Given a multi-qubit Pauli operator, applies the corresponding operation
to a quantum register.

## Input
### pauli
A multi-qubit Pauli operator represented as an array of single-qubit Pauli operators.
### target
Register to apply the given Pauli operation on.

## Example
The following are equivalent:
```qsharp
ApplyPauli([PauliY, PauliZ, PauliX], target);
```
and
```qsharp
Y(target[0]);
Z(target[1]);
X(target[2]);
```
