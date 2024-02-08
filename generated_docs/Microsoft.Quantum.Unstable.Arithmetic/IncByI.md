# operation IncByI(c : Int, ys : Qubit[]) : Unit is Adj + Ctl

## Summary
Increments a little-endian register ys by an integer number c

## Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
Length(ys) = n > 0, c is a Int number, 0 ≤ c < 2ⁿ.
NOTE: Use IncByIUsingIncByLE directly if the choice of implementation
is important.
