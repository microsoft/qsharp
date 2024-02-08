# operation ApplyControlledOnInt<'T>(numberState : Int, oracle : ('T => Unit is Param<1>), controlRegister : Qubit[], target : 'T) : Unit is Adj + Ctl

## Summary
Applies a unitary operation on the target if the control
register state corresponds to a specified nonnegative integer.

## Input
### numberState
A nonnegative integer on which the operation `oracle` should be
controlled.
### oracle
A unitary operation to be controlled.
### target
A target on which to apply `oracle`.
### controlRegister
A quantum register that controls application of `oracle`.

## Remarks
The value of `numberState` is interpreted using a little-endian encoding.

`numberState` must be at most $2^\texttt{Length(controlRegister)} - 1$.
For example, `numberState = 537` means that `oracle`
is applied if and only if `controlRegister` is in the state $\ket{537}$.
