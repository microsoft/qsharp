Let's reuse the `RandomBit` operation from the "Generate A Single Random Bit" exercise again.
You'll generate $N$ random bits by calling `RandomBit` operation $N$ times, and treat the result as a binary notation of the integer you're looking for.
Since the maximum value of the number written with $N$ bits is $2^N - 1$, you don't need to do any extra checks to ensure that the result is within the given range.

@[solution]({
    "id": "random_numbers__random_n_bits_solution",
    "codePath": "Solution.qs"
})
