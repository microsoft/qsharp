# operation MeasureInteger(target : Qubit[]) : Int

## Summary
Measures the content of a quantum register and converts
it to an integer. The measurement is performed with respect
to the standard computational basis, i.e., the eigenbasis of `PauliZ`.

## Input
### target
A quantum register in the little-endian encoding.

## Output
An unsigned integer that contains the measured value of `target`.

## Remarks
This operation resets its input register to the |00...0> state,
suitable for releasing back to a target machine.
