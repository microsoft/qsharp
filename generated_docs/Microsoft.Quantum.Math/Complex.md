---
uid Microsoft.Quantum.Math.Complex
title: Complex user defined type
ms.date: todo
ms.topic: managed-reference
qsharp.kind: udt
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: Complex
qsharp.summary: Represents a complex number by its real and imaginary components.
The first element of the tuple is the real component,
the second one - the imaginary component.
---

# Complex user defined type

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

Represents a complex number by its real and imaginary components.
The first element of the tuple is the real component,
the second one - the imaginary component.
```qsharp
newtype Complex = (Real: Double, Imag: Double)
```

## Summary
Represents a complex number by its real and imaginary components.
The first element of the tuple is the real component,
the second one - the imaginary component.

## Example
The following snippet defines the imaginary unit 𝑖 = 0 + 1𝑖:
```qsharp
let imagUnit = Complex(0.0, 1.0);
```
