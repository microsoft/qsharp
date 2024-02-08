# DrawMany operation

`operation DrawMany<'TInput, 'TOutput>(op : ('TInput => 'TOutput is Param<2>), nSamples : Int, input : 'TInput) : 'TOutput[]`

## Summary
Repeats an operation for a given number of samples, collecting its outputs
in an array.

## Input
### op
The operation to be called repeatedly.
### nSamples
The number of samples of calling `op` to collect.
### input
The input to be passed to `op`.

## Type Parameters
### TInput
The type of input expected by `op`.
### TOutput
The type of output returned by `op`.

## Example
The following samples an alternating array of results.
```qsharp
use qubit = Qubit();
let results = Microsoft.Quantum.Arrays.DrawMany(q => {X(q); M(q)}, 3, qubit);
```
