### Solution

We can reuse `RandomNBits` operation from [Exercise 3](#Exercise-3:-Generate-a-number-of-arbitrary-size).

We'll generate an N-bit random number by calling `RandomNBits` operation, where N is the bitsize of $max - min$. We can repeat this process until the result is less than or equal than $max - min$, and return that number plus $min$.
