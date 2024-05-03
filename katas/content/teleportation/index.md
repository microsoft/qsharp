# Teleportation

@[section]({
    "id": "teleportation__overview",
    "title": "Overview"
})

Teleportation quantum kata is a series of exercises designed to get you familiar with programming in Q#. It covers the quantum teleportation protocol which allows you to communicate a quantum state using only classical communication and previously shared quantum entanglement.

- Teleportation is described in <a href="https://en.wikipedia.org/wiki/Quantum_teleportation">this Wikipedia article</a>.
- An interactive demonstration can be found <a href="http://demonstrations.wolfram.com/QuantumTeleportation/">on the Wolfram Demonstrations Project</a>.
Each task is wrapped in one operation preceded by the description of the task. Your goal is to fill in the blank (marked with // ... comment) with some Q# code that solves the task. To verify your answer, run the cell

@[section]({
    "id": "teleportation__standard_teleportation",
    "title": "Standard Teleportation"
})

We split the teleportation protocol into several steps:

- Preparation (creating the entangled pair of qubits that are sent to Alice and Bob).
- Sending the message (Alice's task): Entangling the message qubit with Alice's qubit and extracting two classical bits to be sent to Bob.
- Reconstructing the message (Bob's task): Using the two classical bits Bob received from Alice to get Bob's qubit into the state in which the message qubit had been originally. Finally, we compose these steps into the complete teleportation protocol.

@[exercise]({
    "id": "teleportation__entangled_pair",
    "title": "Entangled Pair",
    "path": "./entangled_pair/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})
