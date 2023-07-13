### Solution

Use a combination of the `S` and `T` operations from the `Microsoft.Quantum.Instrinsic` namespace.

```qsharp
operation ThreeQuartersPiPhase(q : Qubit) : Unit is Adj + Ctl {
    S(q);
    T(q);
}
```
