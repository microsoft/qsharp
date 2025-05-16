// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

// This file gives the order, title, and default shots for each sample

/** @type {Array<{title: string; file: string; shots: number; omitFromTests?: boolean}>} */
export default [
    { title: "Hello World in OpenQASM", file: "./OpenQasmHelloWorld.qasm", shots: 100 },
    { title: "Random Number Generator", file: "./RandomNumber.qasm", shots: 1000 },
    { title: "Bell Pair Creation", file: "./BellPair.qasm", shots: 1000 },
    { title: "Bernstein-Vazirani Algorithm", file: "./BernsteinVazirani.qasm", shots: 10 },
    { title: "Grover Search Algorithm", file: "./Grover.qasm", shots: 10 },
]
