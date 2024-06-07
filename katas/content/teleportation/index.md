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
    "title": "Send Message (Alice's Task)",
    "path": "./send_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_message",
    "title": "Reconstruct Message (Bob's Task)",
    "path": "./reconstruct_message/",
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

@[exercise]({
    "id": "teleportation__prepare_and_send_message",
    "title": "Prepare Message and Send It",
    "path": "./prepare_and_send_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_and_measure_message",
    "title": "Reconstruct Message and Measure It",
    "path": "./reconstruct_and_measure_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[section]({
    "id": "teleportation__testing_standard_teleportation",
    "title": "Testing standard quantum teleportation"
})

In this lesson, your goal is to put together the code from the previous exercises to teleport the states $\ket{0}$ and $\ket{1}$, as well as superposition states $\frac{1}{2}(\ket{0}+\ket{1})$, $\frac{1}{2}(\ket{0}-\ket{1})$, $\frac{1}{2}(\ket{0}+i\ket{1})$ and $\frac{1}{2}(\ket{0}-i\ket{1})$, and to verify that teleportation succeeds each time.

> This is an open-ended task that is not tested automatically, unlike the previous exercises. Follow the suggestions in the comments to write your code and test it!

@[example]({
    "id": "teleportation__testing_standard_teleportation_example", 
    "codePath": "./examples/TestingStandardTeleportation.qs"
})

@[section]({
    "id": "teleportation__three_parties",
    "title": "Teleportation with Three Parties"
})

There are multiple variants of teleportation protocol that involve more than two parties. In this lesson, we'll take a look at two of them:

- Entanglement swapping allows us to propagate entanglement across space, enabling protocols such as quantum repeater.
- In teleportation with three entangled qubits, a state is transferred from Alice to a third participant Charlie, but this may only be accomplished if Charlie has the trust of the second participant Bob.

@[exercise]({
    "id": "teleportation__entanglement_swapping",
    "title": "Entanglement Swapping",
    "path": "./entanglement_swapping/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__entangled_trio",
    "title": "Entangled Trio",
    "path": "./entangled_trio",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_message_charlie",
    "title": "Reconstruct message (Charlie's task)",
    "path": "./reconstruct_message_charlie",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[section]({
    "id": "teleportation__principle_of_deferred_measurement",
    "title": "Principle of deferred measurement"
})

The principle of deferred measurement claims that measurements can be moved from an intermediate stage of a quantum circuit to the end of the circuit. If the measurement results are used to perform classically controlled operations, they can be replaced by controlled quantum operations.

@[exercise]({
    "id": "teleportation__measurement_free_teleportation",
    "title": "Measurement-free teleportation",
    "path": "./measurement_free_teleportation",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})