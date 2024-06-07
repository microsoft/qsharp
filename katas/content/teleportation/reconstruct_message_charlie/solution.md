Charlie's qubit now contains the information about the amplitudes of the teleported state, but it needs correction based on the classical message received from Alice and Bob to match the teleported state precisely. Since there are three bool values, total of eight possibilities can be there:
- For 000, no change is required.
- For 001, only X correction is required.
- For 010, only X correction is required.
- For 011, no change is required or resultant operation should be Identity.
- For 100, only Z correction is requried.
- For 101, both Z and X correction is requried.
- For 110, both Z and X correction is requried.
- For 111, only Z correction is requried.

To sum up, based on measurement result of `qMessage`, $Z$ gate should be applied while based on the result of `qAlice` and `qBob`, $X$ gate should be used.

@[solution]({
    "id": "teleportation__reconstruct_message_charlie_solution",
    "codePath": "./Solution.qs"
})