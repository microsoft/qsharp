# function RangeReverse(r : Range) : Range

## Summary
Returns a new range which is the reverse of the input range.

## Input
### r
Input range.

## Output
A new range that is the reverse of the given range.

## Remarks
Note that the reverse of a range is not simply `end`..`-step`..`start`, because
the actual last element of a range may not be the same as `end`.
