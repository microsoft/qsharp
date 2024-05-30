Bob's qubit now contains the information about the amplitudes of the teleported state, but it needs correction based on the classical message received for his qubit to match the teleported state precisely:
- For 00, no change is required.
- For 01, only Z correction is required.
- For 10, only X correction is required.
- For 11, both Z and X correction is requried.

@[solution]({
    "id": "teleportation__reconstruct_the_message_solution",
    "codePath": "./Solution.qs"
})