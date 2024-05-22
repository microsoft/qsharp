# Teleportation

@[section]({
    "id": "teleportation__overview",
    "title": "Overview"
})

Quantum teleportation protocol allows us to communicate a quantum state using only classical communication and previously shared quantum entanglement.

- Teleportation is described in [this Wikipedia article](https://en.wikipedia.org/wiki/Quantum_teleportation).
- An interactive demonstration can be found [on the Wolfram Demonstrations Project](http://demonstrations.wolfram.com/QuantumTeleportation/).

@[section]({
    "id": "teleportation__standard_teleportation",
    "title": "Standard Teleportation"
})

In the standard teleportation protocol, two parties, the sender (typically referred to as Alice) and the receiver (Bob) start by sharing an entangled pair of qubits. The goal of the protocol is for Alice to transfer an unknown quantum state to Bob using their shared qubit pair and a classical communication channel.

We split the teleportation protocol into several steps:

- Preparation (creating the entangled pair of qubits that are sent to Alice and Bob).
- Sending the message (Alice's task): Entangling the message qubit with Alice's qubit and extracting two classical bits to be sent to Bob.
- Reconstructing the message (Bob's task): Using the two classical bits Bob received from Alice to get Bob's qubit into the state in which the message qubit had been originally.
- Finally, we compose these steps into the complete teleportation protocol.

@[exercise]({
    "id": "teleportation__entangled_pair",
    "title": "Entangled Pair",
    "path": "./entangled_pair/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__send_message",
    "title": "Send the message (Alice's task)",
    "path": "./send_the_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_message",
    "title": "Reconstruct the message (Bob's task)",
    "path": "./reconstruct_the_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__standard_teleportation_protocol",
    "title": "Standard Teleportation Protocol",
    "path": "./standard_teleportation_protocol/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})