# Superdense Coding Kata

@[section]({
    "id": "./superdense_coding.ipynb__overview",
    "title": "Overview"
})

**Superdense Coding** quantum kata is a series of exercises designed to get you familiar with programming in Q#.

It covers the superdense coding protocol which allows us to transmit two bits of classical information by sending just one qubit using previously shared quantum entanglement.

- A good description can be found in [the Wikipedia article](https://en.wikipedia.org/wikiSuperdense_coding).
- A great interactive demonstration can be found [on the Wolfram Demonstrations Project](http:/demonstrations.wolfram.com/SuperdenseCoding/).
- Superdense coding protocol is described in Nielsen & Chuang, section 2.3 (pp. 97-98).


We split the superdense coding protocol into several steps, following the description in the [Wikipedia article](https://en.wikipedia.org/wiki/Superdense_coding):

- Preparation (creating the entangled pair of qubits that are sent to Alice and Bob).
- Encoding the message (Alice's task): Encoding the classical bits of the message into the state of Alice's qubit which then is sent to Bob.
- Decoding the message (Bob's task): Using Bob's original qubit and the qubit he received from Alice to decode the classical message sent.
- Finally, we compose those steps into the complete superdense coding protocol.

@[exercise]({
    "id": "./superdense_coding__entangled_pair",
    "title": "Entangled pair",
    "path": "entangled_pair",
    "qsDependencies": [
        "../KatasLibrary.qs"
    ]
})
