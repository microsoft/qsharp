# RangeStep function

`function RangeStep(r : Range) : Int`

## Summary
Returns the integer that specifies how the next value of a range is calculated.

## Input
### r
Input range.

## Output
The defined step value of the given range.

## Remarks
A range expression's first element is `start`,
its second element is `start+step`, third element is `start+step+step`, etc.,
until `end` is passed.
