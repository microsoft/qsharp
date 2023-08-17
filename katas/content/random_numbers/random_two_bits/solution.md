Let's reuse the `RandomBit` operation from the "Generate A Single Random Bit" exercise.
We can generate two random bits by calling the `RandomBit` operation twice, multiply the most significant bit by 2 and add the second random bit to generate a random two-bit number.

@[solution]({
    "id": "random_two_bits_solution",
    "codePath": "solution.qs"
})
