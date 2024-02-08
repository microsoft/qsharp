# operation IncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

## Summary
Increments a little-endian register ys by a little-endian register xs

## Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
NOTE: Use operations like RippleCarryCGIncByLE directly if
the choice of implementation is important.
