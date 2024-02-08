# ApplyToEachA operation

`operation ApplyToEachA<'T>(singleElementOperation : ('T => Unit is Param<1>), register : 'T[]) : Unit is Adj`

## Summary
Applies an operation to each element in a register.
The modifier `A` indicates that the single-element operation is adjointable.

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

## See Also
- Microsoft.Quantum.Canon.ApplyToEach
