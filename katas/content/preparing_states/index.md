# Preparing Quantum States

@[section]({
    "id": "preparing_states__overview",
    "title": "Overview"
})

This kata is designed to get you familiar with quantum state preparation and to help you practice using gates to change quantum states.

**This kata covers the following topics:**

- Using basic single-qubit and multi-qubit gates to prepare quantum states
- Manipulating superposition states
- Flow control and recursion in Q#

**What you should know to start working on this kata:**

- Dirac notation for single-qubit and multi-qubit quantum systems
- Basic single-qubit and multi-qubit gates

If you need a refresher on these topics, you can check out the previous katas.

@[section]({
    "id": "preparing_states__even_superpositions",
    "title": "Even Superpositions"
})

In this lesson, you'll practice preparing states that are even superpositions of specific sets of basis states, possibly with some relative phases on some of the basis states. These exercises can be solved using some sequences of $H$ and $X$ gates, their controlled variants, and some phase-introducing gates.

@[exercise]({
    "id": "preparing_states__plus_state",
    "title": "Plus State",
    "path": "./plus_state/"
})

@[exercise]({
    "id": "preparing_states__minus_state",
    "title": "Minus State",
    "path": "./minus_state/"
})

@[exercise]({
    "id": "preparing_states__even_sup_two_qubits",
    "title": "All Two-Qubit Basis Vectors",
    "path": "./even_sup_two_qubits/"
})

@[exercise]({
    "id": "preparing_states__even_sup_two_qubits_phase_flip",
    "title": "All Two-Qubit Basis Vectors with Phase Flip",
    "path": "./even_sup_two_qubits_phase_flip/"
})

@[exercise]({
    "id": "preparing_states__even_sup_two_qubits_complex_phases",
    "title": "All Two-Qubit Basis Vectors with Complex Phases",
    "path": "./even_sup_two_qubits_complex_phases/"
})

@[exercise]({
    "id": "preparing_states__bell_state",
    "title": "Bell State",
    "path": "./bell_state/"
})

@[exercise]({
    "id": "preparing_states__all_bell_states",
    "title": "All Bell States",
    "path": "./all_bell_states/"
})

@[exercise]({
    "id": "preparing_states__ghz_state",
    "title": "Greenberger-Horne-Zellinger State",
    "path": "./greenberger_horne_zeilinger/"
})

@[exercise]({
    "id": "preparing_states__all_basis_vectors",
    "title": "All N-Qubit Basis Vectors",
    "path": "./all_basis_vectors/"
})

@[exercise]({
    "id": "preparing_states__even_odd",
    "title": "Even or Odd Basis Vectors",
    "path": "./even_odd/"
})

@[exercise]({
    "id": "preparing_states__zero_and_bitstring",
    "title": "Zero and Bitstring",
    "path": "./zero_and_bitstring/"
})

@[exercise]({
    "id": "preparing_states__two_bitstrings",
    "title": "Two Bitstrings",
    "path": "./two_bitstrings/"
})

@[exercise]({
    "id": "preparing_states__four_bitstrings",
    "title": "Four Bit Strings",
    "path": "./four_bitstrings/"
})

@[exercise]({
    "id": "preparing_states__parity_bitstrings",
    "title": "Same Parity Bit Strings",
    "path": "./parity_bitstrings/"
})


@[section]({
    "id": "preparing_states__arbitrary_rotations",
    "title": "Arbitrary Rotations"
})

In this lesson, you'll practice preparing more interesting states that require the use of arbitrary rotation gates. 

> Some of the alternative solutions rely on clever tricks such as partial measurement of the system. 
> To fully appreciate them, come back to this kata after you've familiarized yourself with quantum measurements!

@[exercise]({
    "id": "preparing_states__unequal_superposition",
    "title": "Unequal Superposition",
    "path": "./unequal_superposition/"
})

@[exercise]({
    "id": "preparing_states__controlled_rotation",
    "title": "Controlled Rotation",
    "path": "./controlled_rotation/"
})

@[exercise]({
    "id": "preparing_states__three_states_two_qubits",
    "title": "Three Two-Qubit Basis States",
    "path": "./three_states_two_qubits/"
})

@[exercise]({
    "id": "preparing_states__three_states_two_qubits_phases",
    "title": "Three Two-Qubit Basis States with Complex Phases",
    "path": "./three_states_two_qubits_phases/"
})

@[exercise]({
    "id": "preparing_states__hardy_state",
    "title": "Hardy State",
    "path": "./hardy_state/"
})

@[exercise]({
    "id": "preparing_states__wstate_power_of_two",
    "title": "W State on Power of Two Number of Qubits",
    "path": "./wstate_power_of_two/"
})

@[exercise]({
    "id": "preparing_states__wstate_arbitrary",
    "title": "W State on Arbitrary Number of Qubits",
    "path": "./wstate_arbitrary/"
})


@[section]({
    "id": "preparing_states__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to use the basic quantum computing gates to prepare quantum states.
