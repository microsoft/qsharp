# operation ApplyXorInPlaceL(value : BigInt, target : Qubit[]) : Unit is Adj + Ctl

## Summary
Applies a bitwise-XOR operation between a classical integer and an
integer represented by a register of qubits.

## Description
Applies `X` operations to qubits in a little-endian register based on
1 bits in an integer.

Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
then `ApplyXorInPlace` performs an operation given by the following map:
|y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
