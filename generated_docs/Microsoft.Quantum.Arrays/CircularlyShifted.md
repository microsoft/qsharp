# function CircularlyShifted<'T>(stepCount : Int, array : 'T[]) : 'T[]

## Summary
Shift an array circularly left or right by a specific step size.

## Type Parameters
### 'T
The type of the array elements.

## Input
### stepCount
The amount of positions by which the array elements will be shifted.
If this is positive, `array` is circularly shifted to the right.
If this is negative, `array` is circularly shifted to the left.
### array
Array to be circularly shifted.

## Output
An array `output` that is the `array` circularly shifted to the right or left
by the specified step size.

## Example
```qsharp
let array = [10, 11, 12];
// The following line returns [11, 12, 10].
let output = CircularlyShifted(2, array);
// The following line returns [12, 10, 11].
let output = CircularlyShifted(-2, array);
```
