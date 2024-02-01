# Summary
Converts a given integer to an equivalent double-precision floating-point number.
---
function IntAsDouble(number : Int) : Double

---

# Summary
Converts a given integer to an equivalent big integer.
---
function IntAsBigInt(number : Int) : BigInt

---

# Summary
Produces a non-negative integer from a string of bits in little endian format.

# Input
## bits
Bits in binary representation of number.
---
function BoolArrayAsInt(bits : Bool[]) : Int

---

# Summary
Produces a binary representation of a non-negative integer, using the
little-endian representation for the returned array.

# Input
## number
A non-negative integer to be converted to an array of Boolean values.
## bits
The number of bits in the binary representation of `number`.

# Output
An array of Boolean values representing `number`.

# Remarks
The input `bits` must be non-negative.
The input `number` must be between 0 and 2^bits - 1.
---
function IntAsBoolArray(number : Int, bits : Int) : Bool[]

---

# Summary
Converts an array of Boolean values into a non-negative BigInt, interpreting the
array as a binary representation in little-endian format.

# Input
## boolArray
An array of Boolean values representing the binary digits of a BigInt.

# Output
A BigInt represented by `boolArray`.

# Remarks
The function interprets the array in little-endian format, where the first
element of the array represents the least significant bit.
The input `boolArray` should not be empty.
---
function BoolArrayAsBigInt(boolArray : Bool[]) : BigInt

---

# Summary
Produces a binary representation of a non-negative BigInt, using the
little-endian representation for the returned array.

# Input
## number
A non-negative BigInt to be converted to an array of Boolean values.
## bits
The number of bits in the binary representation of `number`.

# Output
An array of Boolean values representing `number`.

# Remarks
The input `bits` must be non-negative.
The input `number` must be between 0 and 2^bits - 1.
---
function BigIntAsBoolArray(number : BigInt, bits : Int) : Bool[]

---

# Summary
Converts a complex number of type `Complex` to a complex
number of type `ComplexPolar`.

# Input
## input
Complex number c = x + y𝑖.

# Output
Complex number c = r⋅e^(t𝑖).
---
function ComplexAsComplexPolar(input : Complex) : ComplexPolar

---

# Summary
Converts a complex number of type `ComplexPolar` to a complex
number of type `Complex`.

# Input
## input
Complex number c = r⋅e^(t𝑖).

# Output
Complex number c = x + y𝑖.
---
function ComplexPolarAsComplex(input : ComplexPolar) : Complex
