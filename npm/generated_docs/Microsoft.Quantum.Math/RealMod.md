---
uid: Microsoft.Quantum.Math.RealMod
title: RealMod function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: RealMod
qsharp.summary: Computes the modulus between two real numbers.
---

# RealMod function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function RealMod(value : Double, modulo : Double, minValue : Double) : Double
```

## Summary
Computes the modulus between two real numbers.

## Input
### value
A real number x to take the modulus of.
### modulo
A real number to take the modulus of x with respect to.
### minValue
The smallest value to be returned by this function.

## Example
```qsharp
    // Returns 3 π / 2.
    let y = RealMod(5.5 * PI(), 2.0 * PI(), 0.0);
    // Returns -1.2, since +3.6 and -1.2 are 4.8 apart on the real line,
    // which is a multiple of 2.4.
    let z = RealMod(3.6, 2.4, -1.2);
```
