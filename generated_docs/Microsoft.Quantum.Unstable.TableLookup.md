# operation Select(data : Bool[][], address : Qubit[], target : Qubit[]) : Unit is Adj + Ctl

## Summary
Performs table lookup using a SELECT network

## Description
Assuming a zero-initialized `target` register, this operation will
initialize it with the bitstrings in `data` at indices according to the
computational values of the `address` register.

## Input
### data
The classical table lookup data which is prepared in `target` with
respect to the state in `address`. The length of data must be less than
2ⁿ, where 𝑛 is the length of `address`. Each entry in data must have
the same length that must be equal to the length of `target`.
### address
Address register
### target
Zero-initialized target register

## Remarks
The implementation of the SELECT network is based on unary encoding as
presented in [1].  The recursive implementation of that algorithm is
presented in [3].  The adjoint variant is optimized using a
measurement-based unlookup operation [3]. The controlled adjoint variant
is not optimized using this technique.

## References
[1] [arXiv:1805.03662](https://arxiv.org/abs/1805.03662)
    "Encoding Electronic Spectra in Quantum Circuits with Linear T
     Complexity"
[2] [arXiv:1905.07682](https://arxiv.org/abs/1905.07682)
    "Windowed arithmetic"
[3] [arXiv:2211.01133](https://arxiv.org/abs/2211.01133)
    "Space-time optimized table lookup"
