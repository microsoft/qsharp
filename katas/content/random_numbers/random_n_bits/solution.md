Let's reuse the `RandomBit` operation from the "Generate A Single Random Bit" exercise again.
We'll generate N random bits by calling `RandomBit` operation N times, and treat the result as a binary notation to convert it into an integer.
Since the maximum value of the number written with N bits is $2^N - 1$, we don't need to do any extra checks to ensure that the result is within the given range.

@[solution]({
    "id": "random_n_bits_solution",
    "codePath": "solution.qs"
})
