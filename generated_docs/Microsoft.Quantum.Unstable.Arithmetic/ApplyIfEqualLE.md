# operation ApplyIfEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl

## Summary
Computes `if x == y { action(target) }`, that is, applies `action` to `target`
if register `x` is equal to the register `y`.
Both qubit registers should be in a little-endian format.
