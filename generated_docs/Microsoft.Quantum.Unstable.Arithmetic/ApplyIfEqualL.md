# operation ApplyIfEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, xs : Qubit[], target : 'T) : Unit is Adj + Ctl

## Summary
Computes `if (c == x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is equal to the little-endian qubit register `x`
