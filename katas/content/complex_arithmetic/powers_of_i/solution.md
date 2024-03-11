When raising $i$ to an integer power, the answer will vary according to a certain pattern.
To figure it out, notice that raising $i$ to the power of $4$ gives: 

$$i^4 = i^2 \cdot i^2 = (-1) \cdot (-1) = 1$$

Thus, when the exponent $n$ is divisible by 4, $i^n$ will always be 1.

When the exponent $n$ is not divisible by 4, you can use the previous observation to see that $i^n = i^{n \mod 4}$.
For an even exponent $n$ that is not divisible by 4 you'll have $i^n = i^2 = -1.$

Here is the complete pattern that arises when raising $i$ to non-negative powers. Note that it is periodic with period $4$.

|Power of $i$ | $i^0$ | $i^1$ | $i^2$ | $i^3$ | $i^4$ | $i^5$ | $i^6$ | $i^7$ | $i^8$ | $\dots$ |
|----|----|----|----|----|----|----|----|----|----|----|
|Result | $1$ | $i$ | $-1$ | $-i$ | $1$ | $i$ | $-1$ | $-i$ | $1$ | $\dots$ |

> `%` is the Q# modulo operator which returns the remainder of a division. For example, `7%2` gives $1$, because $1$ is the remainder of dividing $7$ by $2$.  
> We can use this operator to determine if the exponent $n$ is divisible by $4$.

@[solution]({"id": "complex_arithmetic__powers_of_i_solution", "codePath": "Solution.qs"})
