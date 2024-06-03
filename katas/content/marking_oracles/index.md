# Marking Oracles

@[section]({
    "id": "marking_oracles__overview",
    "title": "Overview"
})

Quantum oracles are a key part of many quantum algorithms that rely on quantum implementation of a classical function. The algorithms' discussions often assume that the quantum oracle that implements the function of interest is provided.
However, in practice implementing a quantum oracle for a specific problem can be quite challenging.

This kata continues the exploration of quantum oracles started in the Oracles kata, offering you a variety of practice problems on implementing marking oracles. If you're not familiar with the concept of quantum oracles, make sure to check out the Oracles kata that introduces the key concepts in this topic and offers a lot of simple practice problems.

**This kata covers the following topics:**

- Implementation of marking oracles for different classical functions

**What you should know to start working on this kata:**

- Fundamental quantum concepts
- Controlled gates
- Oracles, in particular marking oracles

@[exercise]({
    "id": "marking_oracles__kth_bit",
    "title": "K-th Bit",
    "path": "./kth_bit/",
    "qsDependencies": [
        "../KatasLibrary.qs"
    ]
})

@[exercise]({
    "id": "marking_oracles__parity",
    "title": "Parity Function",
    "path": "./parity/",
    "qsDependencies": [
        "../KatasLibrary.qs"
    ]
})

@[exercise]({
    "id": "marking_oracles__product",
    "title": "Product Function",
    "path": "./product/",
    "qsDependencies": [
        "../KatasLibrary.qs"
    ]
})

@[exercise]({
    "id": "marking_oracles__product_negation",
    "title": "Product Function with Negation",
    "path": "./product_negation/",
    "qsDependencies": [
        "../KatasLibrary.qs"
    ]
})

@[section]({
    "id": "marking_oracles__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you got a lot of practice building quantum oracles for different classical functions.
