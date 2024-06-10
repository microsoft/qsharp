If Alice and Bob exchanged a qubit and used the same bases for preparing and measuring it, the bit produced by Bob's measurement would be the same as the one Alice encoded. Thus, they do not need to share the bits they chose or obtained over the classical communication channel. Sharing the bases used for each of the qubit is sufficient to understand if their bits match or not.

To complete this task, we need to perform the following steps:

1. Declare an empty mutable array, let's name it `key`.
2. Decide which bits we can add to our key based on the comparison between bases used by Alice and Bob. You can iterate using an index in the range from $0$ to $N - 1$ and compare the bases in the corresponding positions.
3. Return the required `key`.

@[solution]({
    "id": "key_distribution__shared_key_solution",
    "codePath": "./Solution.qs"
})
