### Solution

Use the `Y` operation from the `Microsoft.Quantum.Instrinsic` namespace.

```qsharp
operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
    // Apply the Pauli Y operation.
    Y(q);
}
```
