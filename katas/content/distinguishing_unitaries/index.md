# Distinguishing Unitaries

@[section]({
    "id": "distinguishing_unitaries__overview",
    "title": "Overview"
})

This kata offers you a series of tasks in which you are given one unitary from the given list and have to figure out which one it is by designing and performing experiments on it.

**This kata covers the following topics:**

- quantum measurements,
- designing experiments to analyze behavior of unitary transformations.

**What you should know to start working on this kata:**

- Dirac notation for single-qubit and multi-qubit quantum systems
- Basic single-qubit and multi-qubit gates
- Quantum measurements and their effect on quantum systems

@[section]({
    "id": "distinguishing_unitaries__single_qubit",
    "title": "Distinguishing Single-Qubit Gates"
})

To start with, let's look at some problems involving distinguishing single-qubit gates.

@[exercise]({
    "id": "distinguishing_unitaries__i_x",
    "title": "Identity or X?",
    "path": "./i_x/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__i_z",
    "title": "Identity or Z?",
    "path": "./i_z/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__z_s",
    "title": "Z or S?",
    "path": "./z_s/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__h_x",
    "title": "Hadamard or X?",
    "path": "./h_x/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__z_minusz",
    "title": "Z or -Z?",
    "path": "./z_minusz/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__rz_r1",
    "title": "Rz or R1?",
    "path": "./rz_r1/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__y_xz",
    "title": "Y or XZ?",
    "path": "./y_xz/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__y_xz_minusy_minusxz",
    "title": "Y or XZ or -Y or -XZ?",
    "path": "./y_xz_minusy_minusxz/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__i_x_y_z",
    "title": "Distinguish Four Pauli Unitaries",
    "path": "./i_x_y_z/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__rz_ry",
    "title": "Rz or Ry?",
    "path": "./rz_ry/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[section]({
    "id": "distinguishing_unitaries__multi_qubit",
    "title": "Distinguishing Multi-Qubit Gates"
})

In this lesson, the exercises focus on distinguishing multi-qubit gates.

@[exercise]({
    "id": "distinguishing_unitaries__ix_cnot",
    "title": "I⊗X or CNOT?",
    "path": "./ix_cnot/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__cnot_direction",
    "title": "CNOT Direction",
    "path": "./cnot_direction/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__cnot_swap",
    "title": "CNOT or SWAP?",
    "path": "./cnot_swap/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[exercise]({
    "id": "distinguishing_unitaries__i_cnot_swap",
    "title": "Distinguish Two-Qubit Unitaries",
    "path": "./i_cnot_swap/",
    "qsDependencies": [
        "../KatasLibrary.qs",
        "./Common.qs"
    ]
})

@[section]({
    "id": "distinguishing_unitaries__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to use measurements and basic quantum computing gates to identify unknown unitaries.
