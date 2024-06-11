Same as in the traditional teleportation protocol, Charlie's qubit now contains the information about the amplitudes $\alpha$ and $\beta$ of the teleported state, but it needs correction based on the classical messages received from Alice and Bob to match the teleported state precisely. Since there are three Boolean values, there is a total of eight possibilities.
- For 000, no change is required.
- For 001, only X correction is required.
- For 010, only X correction is required.
- For 011, no change is required or resultant operation should be Identity.
- For 100, only Z correction is requried.
- For 101, both Z and X correction is requried.
- For 110, both Z and X correction is requried.
- For 111, only Z correction is requried.

To sum up, we need to apply a $Z$ gate if the measurement result of `qMessage` is 1, and an $X$ gate if the measurement results of `qAlice` and `qBob` are different.

@[solution]({
    "id": "teleportation__reconstruct_message_charlie_solution",
    "codePath": "./Solution.qs"
})