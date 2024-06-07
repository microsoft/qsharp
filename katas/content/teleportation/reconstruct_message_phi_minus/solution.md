Bob's side of this protocol can actually be represented as a sequence of two steps:

 1. Transform the Bell pair he shares with Alice back to $\ket{\Phi ^+}$. This can be done by applying the $Z$ gate to Bob's qubit.
 2. Apply the standard teleportation protocol.

The final set of corrections we need is the $Z$ gate, followed by the standard teleportation corrections:
- For 00, only Z correction is required.
- For 01, no change is required.
- For 10, both Z and X correction is required.
- For 11, only X correction is requried.

@[solution]({
    "id": "teleportation__reconstruct_message_phi_minus_solution",
    "codePath": "./Solution.qs"
})