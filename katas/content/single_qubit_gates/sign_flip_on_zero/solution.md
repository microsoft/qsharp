### Solution

Use a combination of the `X`, `Z` and `Y` operations from the `Microsoft.Quantum.Instrinsic` namespace.

```qsharp
operation SignFlipOnZero(q : Qubit) : Unit is Adj + Ctl {
    X(q);
    Z(q);
    Y(q);
}
```
