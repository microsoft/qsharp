# Count function

`function Count<'T>(predicate : ('T -> Bool), array : 'T[]) : Int`

## Summary
Given an array and a predicate that is defined
for the elements of the array, returns the number of elements
an array that consists of those elements that satisfy the predicate.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### predicate
A function from `'T` to Boolean that is used to filter elements.
### array
An array of elements over `'T`.

## Output
The number of elements in `array` that satisfy the predicate.

## Example
```qsharp
 let evensCount = Count(x -> x % 2 == 0, [1, 3, 6, 7, 9]);
// evensCount is 1.
```
