### Solution

Use a combination of the `X` and `H` operations from the `Microsoft.Quantum.Instrinsic` namespace.

```qsharp
operation PrepareMinus(q : Qubit) : Unit is Adj + Ctl {
    X(q);
    H(q);
}
```
