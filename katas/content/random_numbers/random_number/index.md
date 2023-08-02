In the "Generate a number of arbitrary size" exercise, we generated numbers in the range $[0, 2^N-1]$ $(1 \leq N \leq 10)$. Now let's create an operation that will return a random number in the range $[min, max]$. 

**Input:** 
Two integers $min$ and $max$ ($0 \leq min \leq max \leq 2^{10}-1$).

**Goal:** Generate a random number in the range $[min, max]$ with an equal probability of getting each of the numbers in this range.

> Useful Q# documentation: 
> * [`BitSizeI` function](https://docs.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.math.bitsizei)

