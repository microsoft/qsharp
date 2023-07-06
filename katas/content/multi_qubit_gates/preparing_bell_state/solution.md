### Solution

Here's the solution:

```qsharp
operation BellState (qs : Qubit[]) : Unit is Adj + Ctl {
    H(qs[0]);
    CNOT(qs[0], qs[1]);
}
```
