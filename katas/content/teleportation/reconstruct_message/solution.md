Bob's qubit now contains the information about the amplitudes of the teleported state, but it needs correction based on the classical message received for his qubit to match the teleported state precisely. As we've seen in the solution for the previous task, there are four possible measurement outcomes:

- For 00, Bob's qubit ends up in the state $\alpha \ket{0} + \beta \ket{1}$, so no change is required.
- For 01, Bob's qubit ends up in the state $\alpha \ket{0} - \beta \ket{1}$, so we need to apply a $Z$ correction.
- For 10, Bob's qubit ends up in the state $\alpha \ket{1} + \beta \ket{0}$, so we need to apply an $X$ correction.
- For 11, Bob's qubit ends up in the state $\alpha \ket{1} - \beta \ket{0}$, se we need to apply both $Z$ and $X$ corrections.

@[solution]({
    "id": "teleportation__reconstruct_the_message_solution",
    "codePath": "./Solution.qs"
})