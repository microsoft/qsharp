Charlie's qubit now contains the information about the amplitudes of the teleported state, but it needs correction based on the classical message received for his qubit to match the teleported state precisely:
- For 000, no change is required.
- For 001, only Z correction is required.
- For 010, only X correction is required.
- For 011, both Z and X correction is requried.
- For 100, only X correction is requried.
- For 101, both Z and X correction is requried.
- For 110, both Z and X correction is requried.
- For 111, only Z correction is requried.

@[solution]({
    "id": "teleportation__reconstruct_message_charlie_solution",
    "codePath": "./Solution.qs"
})