# Quantum Key Distribution

@[section]({
    "id": "key_distribution__overview",
    "title": "Overview"
})

Quantum key distribution is a type of quantum communication protocol that allows two parties to generate shared secret keys - random strings of bits known only to those two parties. These shared keys can then be used for a variety of different classical cryptographic protocols like encryption or authentication.

Quantum key distribution protocols include two parties, commonly referred to as Alice and Bob, that have two communication channels between them, one quantum channel that allows Alice to send qubits to Bob and one bidirectional classical channel.
The quantum channel in such protocols is usually implemented with photons acting as qubits.
Note that the classical communication channel has to be authenticated, so that both parties can verify that the classical messages they receive indeed come from the party they are communicating with and are not tampered with in transit.

@[section]({
    "id": "key_distribution__bb84",
    "title": "BB84 Quantum Key Distribution Protocol"
})

BB84 protocol, named after its inventors Charles H. Bennett and Gilles Brassard and year of publication, is one of the first proposed quantum key distribution protocols and probably the most famous one.

BB84 protocol consists of the two main phases:

1. During the first phase, Alice prepares individual qubits following a certain procedure and then sends them to Bob via the quantum channel to be measured. Alice takes notes of the classical decisions she made when preparing the qubits, and Bob - of his decisions and the measurement results.

2. The second phase is entirely classical post-processing and communication: Alice and Bob discuss their data from the first phase and extract a classical, random bit string they can use as a shared key.

Let's start by looking at how Alice prepares her qubits for sending them to Bob.

Alice has two choices for each qubit, which basis to prepare it in, and what bit value she wants to encode.
In the first basis, the computational basis, Alice prepares the states $\ket{0}$ and $\ket{1}$ where $\ket{0}$ represents the key bit value `0` and $\ket{1}$ represents the key bit value `1`.
The second basis, Hadamard basis (sometimes also called the diagonal basis), uses the states $\ket{+} = \frac{1}{\sqrt2}(\ket{0} + \ket{1})$ to represent the key bit value `0`, and $\ket{-} = \frac{1}{\sqrt2}(\ket{0} - \ket{1})$ to represent the key bit value `1`.

<table>
  <tr>
    <th style="text-align:center">Basis</th>
    <th style="text-align:center">Bit 0</th>
    <th style="text-align:center">Bit 1</th>    
  </tr>
  <tr>
    <td style="text-align:center">Computational basis</td>
    <td style="text-align:center">$\ket{0}$</td>
    <td style="text-align:center">$\ket{1}$</td>
  </tr>
  <tr>
    <td style="text-align:center">Hadamard basis</td>
    <td style="text-align:center">$\ket{+}$</td>
    <td style="text-align:center">$\ket{-}$</td>
  </tr>
</table>

The bases used in the protocol are selected such that if an eavesdropper tries to measure a qubit in transit and chooses the wrong basis, then they just get a 0 or 1 measurement result with equal probability.

Alice has to make two random choices for each qubit she prepares, one for which basis to prepare in, and the other for what bit value she wants to send, choosing each option with $50\%$ probability.
If Alice decides to send $N$ qubits, she needs to make $2N$ random choices, usually implemented as two arrays of $N$ choices each.

Once Bob receives the qubits from Alice, he needs to decide in which basis, computational or Hadamard, to measure each of them, and these decisions are also random, with each basis chosen with $50\%$ probability.

Finally, at the end of the first phase of the protocol Alice has a list of the bit values she sent as well as what basis she prepared each qubit in, and Bob has a list of bases he used to measure each qubit. 

To extract the shared key, they need to figure out when they both used the same basis, and toss the data from qubits where they used different bases. If Alice and Bob did not use the same basis to prepare and measure the qubits in, the measurement results Bob got will be just random bits with $50\%$ probability for both the `Zero` and `One` outcomes. But if they used the same basis, Bob's measurement result will match the bit Alice sent.

This means that by exchanging information about the bases Alice and Bob used for preparation and measurements via a public classical communication channel, they can deduce which parts of their lists of bits they kept private are identical, and use them as their shared key!

Now that we've learned the theory behind the BB84 protocol, let's implement its steps to see it in action!

@[exercise]({
    "id": "key_distribution__random_array",
    "title": "Generate Random Array",
    "path": "./random_array/",
    "qsDependencies": []
})

@[exercise]({
    "id": "key_distribution__prepare_qubits",
    "title": "Prepare Qubits (Alice's Task)",
    "path": "./prepare_qubits/",
    "qsDependencies": [
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "key_distribution__measure_qubits",
    "title": "Measure Qubits (Bob's Task)",
    "path": "./measure_qubits/",
    "qsDependencies": [
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "key_distribution__shared_key",
    "title": "Generate the Shared Key",
    "path": "./shared_key/",
    "qsDependencies": [
        "./Common.qs"
    ]
})


@[section]({
    "id": "key_distribution__bb84_e2e",
    "title": "BB84 Protocol End-to-End"
})

In this lesson, your goal is to put together the code from the previous exercises to simulate the complete BB84 protocol, from Alice choosing her bits and sending qubits to Bob to them figuring out the shared key.

> This is an open-ended task that is not tested automatically, unlike the previous exercises. Follow the suggestions in the comments to write your code and test it!

@[example]({"id": "key_distribution__bb84_demo", "codePath": "./examples/BB84Demo.qs"})


@[section]({
    "id": "key_distribution__bb84_eavesdropper",
    "title": "Detecting an Eavesdropper"
})

Now, let's consider adding an eavesdropper Eve in the protocol.

Eve can intercept a qubit from the quantum channel that Alice and Bob are using. 
She can try to get some information from it by measuring it. Then she prepares a new qubit and sends it back to the channel for Bob to receive. 
Eve hopes that if she got lucky with her measurement, that when Bob measures the qubit he doesn't get an error so she won't be caught!

How can Alice and Bob detect an eavesdropper? 

To do this, they need to reveal a part of their shared key publicly to check that they both got the same bits on the qubits for which they used the same bases. If Eve doesn't guess which basis to use for measurement, she'll introduce an error in the protocol by sending $\ket{+}$ or $\ket{-}$ when the computational basis was used or $\ket{0}$ or $\ket{1}$ when the Hadamard basis was used, thus sometimes causing Bob's measurement result differ from the bit Alice encoded even when their bases matched. By comparing their results on a random subset of bits, Alice and Bob will be able to find such discrepancies and detect Eve's presence.

> Feel free to experiment with the code above to introduce an eavesdropper into it and to model the information they can get about the key and the way they can be detected.


@[section]({
    "id": "key_distribution__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to implement the simplest quantum key distribution protocol - BB84.
