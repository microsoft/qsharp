You can reuse the `RandomBit` and `RandomNBits` operations from earlier exercises.

You'll generate an $N$-bit random number by calling the `RandomNBits` operation, where $N$ is the bitsize of $max - min$. You can repeat this process until the result is less than or equal than $max - min$, and return that number plus $min$.

@[solution]({
    "id": "random_numbers__random_number_solution",
    "codePath": "Solution.qs"
})
