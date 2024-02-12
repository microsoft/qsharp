---
uid Microsoft.Quantum.Math.ComplexPolar
title: ComplexPolar user defined type
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: udt
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: ComplexPolar
qsharp.summary: Represents a complex number in polar form.
The polar representation of a complex number is c = r⋅𝑒^(t𝑖).
---

# ComplexPolar user defined type

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
newtype ComplexPolar = (Magnitude: Double, Argument: Double)
```

## Summary
Represents a complex number in polar form.
The polar representation of a complex number is c = r⋅𝑒^(t𝑖).

## Named Items
### Magnitude
The absolute value r>0 of c.
### Argument
The phase t ∈ ℝ of c.
