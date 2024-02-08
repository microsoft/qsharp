# IncByLUsingIncByLE operation

`operation IncByLUsingIncByLE(adder : ((Qubit[], Qubit[]) => Unit is Param<0>), c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl`

## Summary
Increments a little-endian register ys by a BigInt number c
using provided adder.

## Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register
Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
