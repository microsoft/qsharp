# Complex User-Defined Type

`newtype Complex = (Real: Double, Imag: Double)`

## Summary
Represents a complex number by its real and imaginary components.
The first element of the tuple is the real component,
the second one - the imaginary component.

## Example
The following snippet defines the imaginary unit 𝑖 = 0 + 1𝑖:
```qsharp
let imagUnit = Complex(0.0, 1.0);
```
