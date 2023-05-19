// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

// This file gives the order, title, and default shots for each sample

/** @type {Array<{title: string; file: string; shots: number}>} */
export default [
    {title: "Minimal", file: "Minimal.qs", shots: 100},
    {title: "Bell state", file: "BellState.qs", shots: 100},
    {title: "Teleportation", file: "Teleportation.qs", shots: 10},
    {title: "Random numbers", file: "qrng.qs", shots: 1000},
    {title: "Deutsch-Jozsa", file: "DeutschJozsa.qs", shots: 1},
    {title: "Bernstein–Vazirani", file: "BernsteinVazirani.qs", shots: 1},
    {title: "Grover's search", file: "Grover.qs", shots: 100},
    {title: "Shor", file: "Shor.qs", shots: 1},
];
