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
    "title": "Prepare state and send it as message (Alice's task)",
    "path": "./prepare_and_send_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_and_measure_message",
    "title": "Reconstruct and measure the message state (Bob's task)",
    "path": "./reconstruct_and_measure_message/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[section]({
    "id": "teleportation__different_entanglement_pair",
    "title": "Teleportation using different entangled pair"
})

In this section we will take a look at the changes in the reconstruction process (Bob's task) if the qubits shared between Alice and Bob are entangled in a different state. Alice's part of the protocol remains the same in all exercises.

As a reminder, the standard teleportation protocol requires shared qubits in state $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}}(\ket{00} + \ket{11})$.

In each exercise, the inputs are:

- Bob's part of the entangled pair of qubits `qBob`.
- The tuple of classical bits received from Alice, in the format used in Send Message exercise.

The goal is to transform Bob's qubit `qBob` into the state in which the message qubit had been originally.

@[exercise]({
    "id": "teleportation__reconstruct_and_message_phi_minus",
    "title": "Reconstruct the message with entangled qubits state |Φ⁻⟩ = (|00⟩ - |11⟩) / sqrt(2)",
    "path": "./reconstruct_message_phi_minus",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_and_message_psi_plus",
    "title": "Reconstruct the message with entangled qubits state |Ψ⁺⟩ = (|01⟩ + |10⟩) / sqrt(2)",
    "path": "./reconstruct_message_psi_plus",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "teleportation__reconstruct_and_message_psi_minus",
    "title": "Reconstruct the message with entangled qubits state |Ψ⁻⟩ = (|01⟩ - |10⟩) / sqrt(2)",
    "path": "./reconstruct_message_psi_minus",
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

@[section]({
    "id": "teleportation__three_entangled_qubits",
    "title": "Teleportation with three entangled qubits"
})

Quantum teleportation using entangled states other than Bell pairs is also feasible. Here we look at just one of many possible schemes - in it a state is transferred from Alice to a third participant Charlie, but this may only be accomplished if Charlie has the trust of the second participant Bob.

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