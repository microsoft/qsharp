# function BoolArrayAsBigInt(boolArray : Bool[]) : BigInt

## Summary
Converts an array of Boolean values into a non-negative BigInt, interpreting the
array as a binary representation in little-endian format.

## Input
### boolArray
An array of Boolean values representing the binary digits of a BigInt.

## Output
A BigInt represented by `boolArray`.

## Remarks
The function interprets the array in little-endian format, where the first
element of the array represents the least significant bit.
The input `boolArray` should not be empty.
