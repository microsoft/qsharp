# function Any<'T>(predicate : ('T -> Bool), array : 'T[]) : Bool

## Summary
Given an array and a predicate that is defined
for the elements of the array, checks if at least one element of
the array satisfies the predicate.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### predicate
A function from `'T` to `Bool` that is used to check elements.
### array
An array of elements over `'T`.

## Output
A `Bool` value of the OR function of the predicate applied to all elements.

## Example
```qsharp
let anyEven = Any(x -> x % 2 == 0, [1, 3, 6, 7, 9]);
```
