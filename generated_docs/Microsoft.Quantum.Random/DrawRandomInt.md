# operation DrawRandomInt(min : Int, max : Int) : Int

## Summary
Draws a random integer in a given inclusive range.

## Input
### min
The smallest integer to be drawn.
### max
The largest integer to be drawn.

## Output
An integer in the inclusive range from `min` to `max` with uniform
probability.

## Remarks
Fails if `max < min`.

## Example
The following Q# snippet randomly rolls a six-sided die:
```qsharp
let roll = DrawRandomInt(1, 6);
```
