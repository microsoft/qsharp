**Inputs:**

1. Real numbers $\alpha$ and $\beta$ such that $\alpha^2 + \beta^2 = 1$.
2. A qubit in state $\ket{0}$.

**Goal:** Use a rotation gate to transform the qubit into state $\alpha\ket{0} -i\beta\ket{1}$.

> You'll probably need functions from the `Std.Math` namespace, specifically <a href="https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.math/arctan2" target="_blank">ArcTan2</a>.
>
> You can assign variables in Q# by using the `let` keyword: `let num = 3;` or `let result = Function(input);`
