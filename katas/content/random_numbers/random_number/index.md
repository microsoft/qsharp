In the "Generate A Number Of Arbitrary Size" exercise, you generated numbers in the range $[0, 2^N-1]$ $(1 \leq N \leq 10)$. Now let's create an operation that will return a random number in the range $[min, max]$. 

**Input:** 
Two integers $min$ and $max$ ($0 \leq min \leq max \leq 2^{10}-1$).

**Goal:** Generate a random number in the range $[min, max]$ with an equal probability of getting each of the numbers in this range.

> Q# namespace `Std.Math` includes useful function `BitSizeI` that calculates the number of bits in the binary representation of the given number.
