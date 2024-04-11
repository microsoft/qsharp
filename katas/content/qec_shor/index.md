# QEC: Bit Flip, Phase Flip, and Shor Codes

@[section]({
    "id": "qec_shor__overview",
    "title": "Overview"
})

This kata introduces you to the basic concepts of quantum error correction using several simple error correction codes.

**This kata covers the following topics:**

- simple models of noise in quantum systems,
- parity measurements in Z and X bases,
- bit flip code - the simplest code that protects qubits from the effects of bit flip noise,
- phase flip code - the simplest code that protects qubits from the effects of phase flip noise,
- Shor code - the simplest code that can protect from an arbitrary error on a single qubit.

**What you should know to start working on this kata:**

- Basic single-qubit and multi-qubit gates
- Single-qubit and multi-qubit quantum measurements and their effect on quantum systems

@[section]({
    "id": "qec_shor__noise",
    "title": "Noise in Classical and Quantum Systems"
})



> For now, we are assuming that the gates and measurements we use for encoding and decoding procedures of a quantum error correction code are perfect and don't introduce any errors themselves. This is a useful assumption to get started with error correction, but in real life all gates and measurements are noisy, so we'll need to modify our approach. 
> *Fault-tolerant quantum computation* handles the more general scenario of performing computations on encoded states in a way that tolerates errors introduced by noisy gates and measurements.


@[section]({
    "id": "qec_shor__parity_measurements",
    "title": "Parity Measurements in Different Bases"
})

- exercise: parity measurement in Z basis
- exercise: parity measurement in X basis


@[section]({
    "id": "qec_shor__bit_flip_code",
    "title": "Bit Flip Code"
})

- exercise: bit flip code: encode
- exercise: bit flip code: detect X error

@[section]({
    "id": "qec_shor__phase_flip_code",
    "title": "Phase Flip Code"
})

- exercise: phase flip code: encode
- exercise: phase flip code: detect Z error


@[section]({
    "id": "qec_shor__shor_code",
    "title": "Shor Code"
})

- exercise: Shor code: encode
- exercise: Shor code: detect all errors
- demo: does Shor code indeed correct all errors?


@[section]({
    "id": "qec_shor__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned the basics of quantum error correction and several simple error-correction codes.
Here are a few key concepts to keep in mind:

- TODO
