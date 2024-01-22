Let's take it a step further and generate an $N$-bit number.

**Input:** An integer $N$ ($1 \le N \le 10$).

**Goal:** Generate a random number in the range $[0, 2^N - 1]$ with an equal probability of getting each of the numbers in this range.

> Useful Q# documentation:
>
> * <a href="https://docs.microsoft.com/azure/quantum/user-guide/language/statements/iterations" target="_blank">for loops</a>
> * <a href="https://docs.microsoft.com/azure/quantum/user-guide/language/typesystem/immutability" target="_blank">mutable variables</a>
> * <a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.powi" target="_blank">exponents</a>

<details>
  <summary><b>Need a hint?</b></summary>
  Remember that you can use previously defined operations to implement your solution. For convenience, the <b>RandomBit</b> operation is already available for you to use in this exercise.
</details>
