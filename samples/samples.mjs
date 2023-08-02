// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

// This file gives the order, title, and default shots for each sample

/** @type {Array<{title: string; file: string; shots: number}>} */
export default [
    {title: "Getting Started", file: "./language/GettingStarted.qs", shots: 100},
    {title: "Bell State", file: "./algorithms/BellState.qs", shots: 100},
    {title: "Teleportation", file: "./algorithms/Teleportation.qs", shots: 1},
    {title: "Random Number Generator", file: "./algorithms/QRNG.qs", shots: 1000},
    {title: "Deutsch-Jozsa", file: "./algorithms/DeutschJozsa.qs", shots: 1},
    {title: "Bernstein–Vazirani", file: "./algorithms/BernsteinVazirani.qs", shots: 1},
    {title: "Grover's search", file: "Grover.qs", shots: 100},
    {title: "Hidden shift", file: "HiddenShift.qs", shots: 1},
    {title: "Shor", file: "Shor.qs", shots: 1},
];
