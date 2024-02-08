# operation ApplyToEach<'T>(singleElementOperation : ('T => Unit is Param<1>), register : 'T[]) : Unit

## Summary
Applies an operation to each element in a register.

## Input
### singleElementOperation
Operation to apply to each element.
### register
Array of elements on which to apply the given operation.

## Type Parameters
### 'T
The target on which the operation acts.

## Example
Prepare a three-qubit |+⟩ state:
```qsharp
use register = Qubit[3];
ApplyToEach(H, register);
```
