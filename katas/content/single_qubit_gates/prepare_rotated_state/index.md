# Preparing a rotated state

**Inputs:**

1. Real numbers $\alpha$ and $\beta$ such that $\alpha^2 + \beta^2 = 1$.
2. A qubit in state $|0\rangle$.

**Goal:** Use a rotation gate to transform the qubit into state $\alpha|0\rangle -i\beta|1\rangle$.

> You will probably need functions from the [Math](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math) namespace, specifically [ArcTan2](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arctan2).
>
> You can assign variables in Q# by using the `let` keyword: `let num = 3;` or `let result = Function(input);`