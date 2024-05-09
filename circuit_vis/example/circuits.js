// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

/* eslint-disable  @typescript-eslint/no-unused-vars */

// Basic example: entanglement with measurement
//export
const entangle = {
    qubits: [{ id: 0 }, { id: 1, numChildren: 1 }],
    operations: [
        {
            gate: 'H',
            targets: [{ qId: 0 }],
        },
        {
            gate: 'X',
            isControlled: true,
            controls: [{ qId: 0 }],
            targets: [{ qId: 1 }],
        },
        {
            gate: 'Measure',
            isMeasurement: true,
            controls: [{ qId: 1 }],
            targets: [{ type: 1, qId: 1, cId: 0 }],
        },
    ],
};

// Sample circuit
//export
const sample = {
    qubits: [{ id: 0, numChildren: 1 }, { id: 1 }, { id: 2 }, { id: 3 }],
    operations: [
        {
            gate: 'Foo',
            conditionalRender: 3,
            targets: [{ qId: 0 }, { qId: 1 }],
            children: [
                {
                    gate: 'H',
                    targets: [{ qId: 1 }],
                },
                {
                    gate: 'RX',
                    displayArgs: '(0.25)',
                    isControlled: true,
                    controls: [{ qId: 1 }],
                    targets: [{ qId: 0 }],
                },
            ],
        },
        {
            gate: 'X',
            targets: [{ qId: 3 }],
        },
        {
            gate: 'X',
            isControlled: true,
            controls: [{ qId: 1 }],
            targets: [{ qId: 2 }, { qId: 3 }],
        },
        {
            gate: 'X',
            isControlled: true,
            controls: [{ qId: 2 }, { qId: 3 }],
            targets: [{ qId: 1 }],
        },
        {
            gate: 'X',
            isControlled: true,
            controls: [{ qId: 1 }, { qId: 3 }],
            targets: [{ qId: 2 }],
        },
        {
            gate: 'X',
            isControlled: true,
            controls: [{ qId: 2 }],
            targets: [{ qId: 1 }, { qId: 3 }],
        },
        {
            gate: 'measure',
            isMeasurement: true,
            controls: [{ qId: 0 }],
            targets: [{ type: 1, qId: 0, cId: 0 }],
        },
        {
            gate: 'ApplyIfElseR',
            isConditional: true,
            controls: [{ type: 1, qId: 0, cId: 0 }],
            targets: [],
            children: [
                {
                    gate: 'H',
                    targets: [{ qId: 1 }],
                    conditionalRender: 1,
                },
                {
                    gate: 'X',
                    targets: [{ qId: 1 }],
                    conditionalRender: 1,
                },
                {
                    gate: 'X',
                    isControlled: true,
                    controls: [{ qId: 0 }],
                    targets: [{ qId: 1 }],
                    conditionalRender: 2,
                },
                {
                    gate: 'Foo',
                    targets: [{ qId: 3 }],
                    conditionalRender: 2,
                },
            ],
        },
        {
            gate: 'SWAP',
            targets: [{ qId: 0 }, { qId: 2 }],
            children: [
                { gate: 'X', isControlled: true, controls: [{ qId: 0 }], targets: [{ qId: 2 }] },
                { gate: 'X', isControlled: true, controls: [{ qId: 2 }], targets: [{ qId: 0 }] },
                { gate: 'X', isControlled: true, controls: [{ qId: 0 }], targets: [{ qId: 2 }] },
            ],
        },
        {
            gate: 'ZZ',
            targets: [{ qId: 1 }, { qId: 3 }],
        },
        {
            gate: 'ZZ',
            targets: [{ qId: 0 }, { qId: 1 }],
        },
        {
            gate: 'XX',
            isControlled: true,
            controls: [{ qId: 0 }],
            targets: [{ qId: 1 }, { qId: 3 }],
        },
        {
            gate: 'XX',
            isControlled: true,
            controls: [{ qId: 2 }],
            targets: [{ qId: 1 }, { qId: 3 }],
        },
        {
            gate: 'XX',
            isControlled: true,
            controls: [{ qId: 0 }, { qId: 2 }],
            targets: [{ qId: 1 }, { qId: 3 }],
        },
    ],
};

// Teleportation algorithm
//export
const teleport = {
    qubits: [
        {
            id: 0,
            numChildren: 1,
        },
        {
            id: 1,
            numChildren: 1,
        },
        {
            id: 2,
        },
    ],
    operations: [
        {
            gate: 'Teleport',
            children: [
                {
                    gate: 'Entangle',
                    children: [
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 1,
                                },
                            ],
                        },
                        {
                            gate: 'X',
                            isControlled: true,
                            controls: [
                                {
                                    qId: 1,
                                },
                            ],
                            targets: [
                                {
                                    qId: 2,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 1,
                        },
                        {
                            qId: 2,
                        },
                    ],
                },
                {
                    gate: 'PrepareMessage',
                    children: [
                        {
                            gate: 'Random',
                            displayArgs: '([0.5, 0.5])',
                            children: [
                                {
                                    gate: 'DrawCategorical',
                                    displayArgs: '([0.5, 0.5])',
                                    children: [
                                        {
                                            gate: 'DrawRandomDouble',
                                            displayArgs: '(0, 1)',
                                            targets: [],
                                        },
                                    ],
                                    targets: [],
                                },
                            ],
                            targets: [],
                        },
                        {
                            gate: 'SetPlus',
                            children: [
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                    ],
                },
                {
                    gate: 'Encode',
                    children: [
                        {
                            gate: 'X',
                            isControlled: true,
                            controls: [
                                {
                                    qId: 0,
                                },
                            ],
                            targets: [
                                {
                                    qId: 1,
                                },
                            ],
                        },
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 0,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                        {
                            qId: 1,
                        },
                    ],
                },
                {
                    gate: 'M',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 0,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 0,
                            cId: 0,
                        },
                    ],
                },
                {
                    gate: 'measure',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 1,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 1,
                            cId: 0,
                        },
                    ],
                },
                {
                    gate: 'Decode',
                    children: [
                        {
                            gate: 'ApplyIfElseR',
                            isConditional: true,
                            controls: [
                                {
                                    type: 1,
                                    qId: 1,
                                    cId: 0,
                                },
                            ],
                            targets: [],
                            children: [
                                {
                                    gate: 'X',
                                    controls: [
                                        {
                                            type: 1,
                                            qId: 1,
                                            cId: 0,
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                    conditionalRender: 2,
                                },
                            ],
                        },
                        {
                            gate: 'ApplyIfElseR',
                            isConditional: true,
                            controls: [
                                {
                                    type: 1,
                                    qId: 0,
                                    cId: 0,
                                },
                            ],
                            targets: [],
                            children: [
                                {
                                    gate: 'Z',
                                    controls: [
                                        {
                                            type: 1,
                                            qId: 0,
                                            cId: 0,
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                    conditionalRender: 2,
                                },
                            ],
                        },
                    ],
                    controls: [
                        {
                            type: 1,
                            qId: 0,
                            cId: 0,
                        },
                        {
                            type: 1,
                            qId: 1,
                            cId: 0,
                        },
                    ],
                    targets: [
                        {
                            qId: 2,
                        },
                    ],
                },
            ],
            targets: [
                {
                    qId: 1,
                },
                {
                    qId: 2,
                },
                {
                    qId: 0,
                },
            ],
        },
    ],
};

// Grover's algorithm
//export
const grover = {
    qubits: [
        {
            id: 0,
            numChildren: 1,
        },
        {
            id: 1,
            numChildren: 1,
        },
        {
            id: 2,
            numChildren: 1,
        },
        {
            id: 3,
            numChildren: 1,
        },
        {
            id: 4,
        },
    ],
    operations: [
        {
            gate: 'GroverSearch',
            children: [
                {
                    gate: 'PrepareSuperposition',
                    children: [
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 0,
                                },
                            ],
                        },
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 1,
                                },
                            ],
                        },
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 2,
                                },
                            ],
                        },
                        {
                            gate: 'H',
                            targets: [
                                {
                                    qId: 3,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                        {
                            qId: 1,
                        },
                        {
                            qId: 2,
                        },
                        {
                            qId: 3,
                        },
                    ],
                },
                {
                    gate: 'GroverIteration',
                    displayArgs: '(Oracle)',
                    children: [
                        {
                            gate: 'Oracle',
                            displayArgs: '(Oracle_6)',
                            children: [
                                {
                                    gate: 'X',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'Oracle_6',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 0,
                                                },
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 4,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'X',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                                {
                                    qId: 4,
                                },
                            ],
                        },
                        {
                            gate: 'Diffuser',
                            children: [
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'ConditionalPhaseFlip',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'Z',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'R',
                                            displayArgs: '(PauliI, 3.141592653589793)',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                        {
                            qId: 1,
                        },
                        {
                            qId: 2,
                        },
                        {
                            qId: 3,
                        },
                        {
                            qId: 4,
                        },
                    ],
                },
                {
                    gate: 'GroverIteration',
                    displayArgs: '(Oracle)',
                    children: [
                        {
                            gate: 'Oracle',
                            displayArgs: '(Oracle_6)',
                            children: [
                                {
                                    gate: 'X',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'Oracle_6',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 0,
                                                },
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 4,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'X',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                                {
                                    qId: 4,
                                },
                            ],
                        },
                        {
                            gate: 'Diffuser',
                            children: [
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'ConditionalPhaseFlip',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'Z',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'R',
                                            displayArgs: '(PauliI, 3.141592653589793)',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                        {
                            qId: 1,
                        },
                        {
                            qId: 2,
                        },
                        {
                            qId: 3,
                        },
                        {
                            qId: 4,
                        },
                    ],
                },
                {
                    gate: 'GroverIteration',
                    displayArgs: '(Oracle)',
                    children: [
                        {
                            gate: 'Oracle',
                            displayArgs: '(Oracle_6)',
                            children: [
                                {
                                    gate: 'X',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'Oracle_6',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 0,
                                                },
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 4,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                                {
                                    gate: 'X',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 4,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                                {
                                    qId: 4,
                                },
                            ],
                        },
                        {
                            gate: 'Diffuser',
                            children: [
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'ConditionalPhaseFlip',
                                    children: [
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'Z',
                                            isControlled: true,
                                            controls: [
                                                {
                                                    qId: 1,
                                                },
                                                {
                                                    qId: 2,
                                                },
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'R',
                                            displayArgs: '(PauliI, 3.141592653589793)',
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 3,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 2,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 1,
                                                },
                                            ],
                                        },
                                        {
                                            gate: 'X',
                                            isAdjoint: true,
                                            targets: [
                                                {
                                                    qId: 0,
                                                },
                                            ],
                                        },
                                    ],
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                        {
                                            qId: 1,
                                        },
                                        {
                                            qId: 2,
                                        },
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 3,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 2,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 1,
                                        },
                                    ],
                                },
                                {
                                    gate: 'H',
                                    isAdjoint: true,
                                    targets: [
                                        {
                                            qId: 0,
                                        },
                                    ],
                                },
                            ],
                            targets: [
                                {
                                    qId: 0,
                                },
                                {
                                    qId: 1,
                                },
                                {
                                    qId: 2,
                                },
                                {
                                    qId: 3,
                                },
                            ],
                        },
                    ],
                    targets: [
                        {
                            qId: 0,
                        },
                        {
                            qId: 1,
                        },
                        {
                            qId: 2,
                        },
                        {
                            qId: 3,
                        },
                        {
                            qId: 4,
                        },
                    ],
                },
                {
                    gate: 'M',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 0,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 0,
                            cId: 0,
                        },
                    ],
                },
                {
                    gate: 'M',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 1,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 1,
                            cId: 0,
                        },
                    ],
                },
                {
                    gate: 'M',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 2,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 2,
                            cId: 0,
                        },
                    ],
                },
                {
                    gate: 'M',
                    isMeasurement: true,
                    controls: [
                        {
                            qId: 3,
                        },
                    ],
                    targets: [
                        {
                            type: 1,
                            qId: 3,
                            cId: 0,
                        },
                    ],
                },
            ],
            targets: [
                {
                    qId: 0,
                },
                {
                    qId: 1,
                },
                {
                    qId: 2,
                },
                {
                    qId: 3,
                },
                {
                    qId: 4,
                },
            ],
        },
    ],
};
