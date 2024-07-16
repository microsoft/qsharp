We need to do three steps to generate an array of random values:

1. Create a mutable array of size $N$ - it will need a default value that can be `false`.
2. For each index from $0$ to $N-1$, choose one of the two values `true` or `false` at random, and assign that value to the array element at that index. We could choose a random bit by allocating a qubit, preparing it in the $\ket{+}$ state, and measuring it. However, we don't need those bits to have quantum origin, so we can use a Q# library operation `DrawRandomInt` instead.
3. Finally, return the generated array.

@[solution]({
    "id": "key_distribution__random_array_solution",
    "codePath": "./Solution.qs"
})
