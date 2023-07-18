### Solution

Let's reuse `RandomBit` operation from [Exercise 1](#Exercise-1:-Generate-a-single-random-bit) again.
We'll generate N random bits by calling `RandomBit` operation N times, and treat the result as a binary notation to convert it into an integer.
Since the maximum value of the number written with N bits is $2^N - 1$, we don't need to do any extra checks to ensure that the result is within the given range.

@[solution]({
"id": "random_n_bits_solution",
"exerciseId": "random_n_bits",
"codePath": "solution.qs"
})
