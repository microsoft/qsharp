// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

// This file gives the order, title, and default shots for each sample

/** @type {Array<{title: string; file: string; shots: number}>} */
export default [
    { title: "Minimal", file: "./language/GettingStarted.qs", shots: 100 },
    { title: "Superposition", file: "./algorithms/Superposition.qs", shots: 100 },
    { title: "Entanglement", file: "./algorithms/Entanglement.qs", shots: 100 },
    { title: "Bell States", file: "./algorithms/BellState.qs", shots: 100 },
    { title: "Teleportation", file: "./algorithms/Teleportation.qs", shots: 1 },
    { title: "Random Bit", file: "./algorithms/RandomBit.qs", shots: 100 },
    { title: "Random Number Generator", file: "./algorithms/QRNGNISQ.qs", shots: 1000 },
    { title: "Random Number Generator (Advanced)", file: "./algorithms/QRNG.qs", shots: 1000 },
    { title: "Deutsch-Jozsa", file: "./algorithms/DeutschJozsaNISQ.qs", shots: 1 },
    { title: "Deutsch-Jozsa (Advanced)", file: "./algorithms/DeutschJozsa.qs", shots: 1 },
    { title: "Bernstein–Vazirani", file: "./algorithms/BernsteinVaziraniNISQ.qs", shots: 1 },
    { title: "Bernstein–Vazirani (Advanced)", file: "./algorithms/BernsteinVazirani.qs", shots: 1 },
    { title: "Grover's search", file: "./algorithms/Grover.qs", shots: 100 },
    { title: "Hidden Shift", file: "./algorithms/HiddenShiftNISQ.qs", shots: 1 },
    { title: "Hidden Shift (Advanced)", file: "./algorithms/HiddenShift.qs", shots: 1 },
    { title: "Shor", file: "./algorithms/Shor.qs", shots: 1 },
];
