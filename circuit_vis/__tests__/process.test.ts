import { Operation, ConditionalRender } from '../src/circuit';
import { RegisterMap, RegisterType, Register } from '../src/register';
import { Metadata, GateType } from '../src/metadata';
import {
    processOperations,
    _groupOperations,
    _alignOps,
    _getClassicalRegStart,
    _opToMetadata,
    _getRegY,
    _splitTargetsY,
    _fillMetadataX,
    _offsetChildrenX,
} from '../src/process';
import {
    minGateWidth,
    startX,
    startY,
    registerHeight,
    gatePadding,
    classicalRegHeight,
    controlBtnOffset,
    groupBoxPadding,
} from '../src/constants';

describe('Testing _groupOperations', () => {
    const registers: RegisterMap = {
        0: { type: RegisterType.Qubit, y: startY },
        1: { type: RegisterType.Qubit, y: startY + registerHeight },
        2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
        3: { type: RegisterType.Qubit, y: startY + registerHeight * 3 },
    };
    test('single qubit gates on 1 qubit register', () => {
        const operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1, 2], [], [], []]);
    });
    test('single qubit gates on multiple qubit registers', () => {
        const operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'T',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 3], [1, 4], [2], []]);
    });
    test('single and multiple qubit(s) gates', () => {
        let operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1, 2], [1], [], []]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1, 2], [1], [], []]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1, 2], [2], [], []]);
        operations = [
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1, 2], [0], [], []]);
        operations = [
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1], [0, 2], [], []]);
    });
    test('multiple qubit gates in ladder format', () => {
        const operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'T',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([
            [0, 5],
            [0, 1, 4, 5],
            [1, 2, 3, 4],
            [2, 3],
        ]);
    });
    test('multiple qubit gates in ladder format with single qubit gate', () => {
        let operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'T',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([
            [0, 8],
            [0, 1, 2, 6, 7, 8],
            [2, 3, 4, 5, 6],
            [3, 5],
        ]);

        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
            {
                gate: 'T',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 2 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([
            [0, 6, 7, 8, 9, 10],
            [0, 1, 2, 4, 5, 10],
            [2, 3, 4],
            [],
        ]);
    });
    test('interleaved multiqubit gates', () => {
        let operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[1], [0, 1], [0, 1], [0]]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [
                    { type: RegisterType.Qubit, qId: 0 },
                    { type: RegisterType.Qubit, qId: 1 },
                ],
                targets: [{ type: RegisterType.Qubit, qId: 3 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [
                    { type: RegisterType.Qubit, qId: 2 },
                    { type: RegisterType.Qubit, qId: 3 },
                ],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([
            [0, 1],
            [0, 1],
            [0, 1],
            [0, 1],
        ]);
        operations = [
            {
                gate: 'Foo',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [
                    { type: RegisterType.Qubit, qId: 0 },
                    { type: RegisterType.Qubit, qId: 2 },
                    { type: RegisterType.Qubit, qId: 3 },
                ],
            },
            {
                gate: 'Bar',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [
                    { type: RegisterType.Qubit, qId: 0 },
                    { type: RegisterType.Qubit, qId: 1 },
                    { type: RegisterType.Qubit, qId: 2 },
                ],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1], [0, 1], [0, 1], [0]]);
    });
    test('classical control gates', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: {
                type: RegisterType.Qubit,
                y: startY + registerHeight,
                children: [{ type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight }],
            },
            2: {
                type: RegisterType.Qubit,
                y: startY + registerHeight + classicalRegHeight * 2,
                children: [{ type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight * 3 }],
            },
            3: { type: RegisterType.Qubit, y: startY + registerHeight + classicalRegHeight * 4 },
        };
        let operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Classical, qId: 2, cId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0, 1], [0], [0], [0]]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Classical, qId: 2, cId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0], [0, 1], [0], [0]]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Classical, qId: 1, cId: 0 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[1], [0, 1], [1], [1]]);
    });
    test('skipped registers', () => {
        let operations: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0], [], [1], []]);
        operations = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Z',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[0], [0, 1], [1], []]);
    });
    test('no qubits', () => {
        const operations: Operation[] = [
            {
                gate: 'NoOp1',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [],
            },
            {
                gate: 'NoOp2',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [],
            },
        ];
        expect(_groupOperations(operations, registers)).toEqual([[], [], [], []]);
    });
    test('empty arguments', () => {
        expect(_groupOperations([], {})).toEqual([]);
    });
});

describe('Testing _alignOps', () => {
    test('single qubit gates', () => {
        const ops: number[][] = [
            [0, 2, 5, 6],
            [1, 3, 4],
        ];
        expect(_alignOps(ops)).toEqual(ops);
    });
    test('correct ordering of single qubit gate after multiqubit gate', () => {
        const ops: number[][] = [
            [0, 1, 3],
            [1, 2],
        ];
        expect(_alignOps(ops)).toEqual([
            [0, 1, 3],
            [null, 1, 2],
        ]);
    });
    test('padding of multiqubit register after single qubit gate', () => {
        const ops: number[][] = [[1], [0, 1]];
        expect(_alignOps(ops)).toEqual([
            [null, 1],
            [0, 1],
        ]);
    });
    test('no padding of single qubit gate after multiqubit gate on different registers', () => {
        const ops: number[][] = [[0, 3], [2], [1, 2]];
        expect(_alignOps(ops)).toEqual([
            [0, 3],
            [null, 2],
            [1, 2],
        ]);
    });
    test('ladder of cnots', () => {
        const ops: number[][] = [
            [0, 4],
            [0, 1, 3, 4],
            [1, 2, 3],
        ];
        expect(_alignOps(ops)).toEqual([
            [0, null, null, null, 4],
            [0, 1, null, 3, 4],
            [null, 1, 2, 3],
        ]);
    });
    test('interleaved multiqubit gates', () => {
        let ops: number[][] = [[0], [0, 1], [0, 1], [1]];
        expect(_alignOps(ops)).toEqual([[0], [0, 1], [0, 1], [null, 1]]);
        ops = [[0], [0], [0, 1], [1], [1], [1]];
        expect(_alignOps(ops)).toEqual([[0], [0], [0, 1], [null, 1], [null, 1], [null, 1]]);
    });
    test('skipped registers', () => {
        let ops: number[][] = [[0], [], [1], []];
        expect(_alignOps(ops)).toEqual([[0], [], [1], []]);
        ops = [[0], [], [1, 2], [2]];
        expect(_alignOps(ops)).toEqual([[0], [], [1, 2], [null, 2]]);
    });
    test('no ops', () => {
        expect(_alignOps([])).toEqual([]);
    });
});

describe('Testing _getClassicalRegStart', () => {
    test('no measurement gates', () => {
        const ops: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'Y',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
        ];
        const idxList: number[][] = [[0, 1]];
        expect(_getClassicalRegStart(ops, idxList)).toEqual([]);
    });
    test('one measurement gate', () => {
        const ops: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            },
        ];
        const idxList: number[][] = [[0, 1]];
        expect(_getClassicalRegStart(ops, idxList)).toEqual([[1, { type: RegisterType.Classical, qId: 0, cId: 0 }]]);
    });
    test('multiple measurement gates', () => {
        const ops: Operation[] = [
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Classical, qId: 1, cId: 0 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            },
        ];
        const idxList: number[][] = [[0, 2, 3], [1]];
        const clsRegs: [number, Register][] = [
            [1, { type: RegisterType.Classical, qId: 0, cId: 0 }],
            [2, { type: RegisterType.Classical, qId: 0, cId: 0 }],
            [0, { type: RegisterType.Classical, qId: 1, cId: 0 }],
        ];
        expect(_getClassicalRegStart(ops, idxList)).toEqual(clsRegs);
    });
});

describe('Testing _opToMetadata', () => {
    test('single qubit gate', () => {
        const op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 1 }],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.X,
            x: 0,
            controlsY: [],
            targetsY: [startY + registerHeight],
            label: 'X',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('isAdjoint gate', () => {
        const op: Operation = {
            gate: 'Foo',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: true,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 1 }],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY + registerHeight],
            label: "Foo'",
            width: 48,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('measure gate', () => {
        const op: Operation = {
            gate: 'M',
            isMeasurement: true,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 0 }],
            targets: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
        };
        const registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
            },
        };
        const metadata: Metadata = {
            type: GateType.Measure,
            x: 0,
            controlsY: [startY],
            targetsY: [startY + classicalRegHeight],
            label: '',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('swap gate', () => {
        const op: Operation = {
            gate: 'SWAP',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.Swap,
            x: 0,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('isControlled swap gate', () => {
        const op: Operation = {
            gate: 'SWAP',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 0 }],
            targets: [
                { type: RegisterType.Qubit, qId: 1 },
                { type: RegisterType.Qubit, qId: 2 },
            ],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
        };
        const metadata: Metadata = {
            type: GateType.Swap,
            x: 0,
            controlsY: [startY],
            targetsY: [startY + registerHeight, startY + registerHeight * 2],
            label: '',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('single qubit unitary gate', () => {
        const op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 0 }],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
        };
        const metadata: Metadata = {
            type: GateType.X,
            x: 0,
            controlsY: [],
            targetsY: [startY],
            label: 'X',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('multiqubit unitary gate', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
        };
        let op: Operation = {
            gate: 'ZZ',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
        };
        let metadata: Metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: 'ZZ',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
        op = {
            gate: 'XX',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 1 },
                { type: RegisterType.Qubit, qId: 2 },
            ],
        };
        metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY + registerHeight, startY + registerHeight * 2],
            label: 'XX',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('isControlled unitary gates', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
            3: { type: RegisterType.Qubit, y: startY + registerHeight * 3 },
        };
        let op: Operation = {
            gate: 'ZZ',
            isMeasurement: false,
            isConditional: false,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 1 }],
            targets: [{ type: RegisterType.Qubit, qId: 0 }],
        };
        let metadata: Metadata = {
            type: GateType.ControlledUnitary,
            x: 0,
            controlsY: [startY + registerHeight],
            targetsY: [startY],
            label: 'ZZ',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
        op = {
            gate: 'XX',
            isMeasurement: false,
            isConditional: false,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 0 }],
            targets: [
                { type: RegisterType.Qubit, qId: 1 },
                { type: RegisterType.Qubit, qId: 2 },
            ],
        };
        metadata = {
            type: GateType.ControlledUnitary,
            x: 0,
            controlsY: [startY],
            targetsY: [startY + registerHeight, startY + registerHeight * 2],
            label: 'XX',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
        op = {
            gate: 'Foo',
            isMeasurement: false,
            isConditional: false,
            isControlled: true,
            isAdjoint: false,
            controls: [
                { type: RegisterType.Qubit, qId: 2 },
                { type: RegisterType.Qubit, qId: 3 },
            ],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
        };
        metadata = {
            type: GateType.ControlledUnitary,
            label: 'Foo',
            x: 0,
            controlsY: [startY + registerHeight * 2, startY + registerHeight * 3],
            targetsY: [startY, startY + registerHeight],
            width: 45,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('single-qubit unitary gates with arguments', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        let op: Operation = {
            gate: 'RX',
            displayArgs: '(0.25)',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 0 }],
        };
        let metadata: Metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY],
            label: 'RX',
            displayArgs: '(0.25)',
            width: 52,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);

        // Test long argument
        op = {
            gate: 'RX',
            displayArgs: "(0.25, 1.0, 'foobar', (3.14, 6.67))",
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 0 }],
        };
        metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY],
            label: 'RX',
            displayArgs: "(0.25, 1.0, 'foobar', (3.14, 6.67))",
            width: 188,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);

        // Test isControlled
        op = {
            gate: 'RX',
            displayArgs: '(0.25)',
            isMeasurement: false,
            isConditional: false,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 1 }],
            targets: [{ type: RegisterType.Qubit, qId: 0 }],
        };
        metadata = {
            type: GateType.ControlledUnitary,
            x: 0,
            controlsY: [startY + registerHeight],
            targetsY: [startY],
            label: 'RX',
            displayArgs: '(0.25)',
            width: 52,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('multi-qubit unitary gates with arguments', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
        };
        let op: Operation = {
            gate: 'U',
            displayArgs: "('foo', 'bar')",
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
        };
        let metadata: Metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: 'U',
            displayArgs: "('foo', 'bar')",
            width: 77,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);

        // Test long argument
        op = {
            gate: 'U',
            displayArgs: "(0.25, 1.0, 'foobar', (3.14, 6.67))",
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
        };
        metadata = {
            type: GateType.Unitary,
            x: 0,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: 'U',
            displayArgs: "(0.25, 1.0, 'foobar', (3.14, 6.67))",
            width: 188,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);

        // Test isControlled
        op = {
            gate: 'U',
            displayArgs: "('foo', 'bar')",
            isMeasurement: false,
            isConditional: false,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Qubit, qId: 1 }],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 2 },
            ],
        };
        metadata = {
            type: GateType.ControlledUnitary,
            x: 0,
            controlsY: [startY + registerHeight],
            targetsY: [startY, startY + registerHeight * 2],
            label: 'U',
            displayArgs: "('foo', 'bar')",
            width: 77,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('classically-controlled gates', () => {
        const op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: true,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
            children: [
                {
                    gate: 'X',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                    conditionalRender: ConditionalRender.OnZero,
                },
                {
                    gate: 'H',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 1 }],
                    conditionalRender: ConditionalRender.OnOne,
                },
            ],
        };
        const registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
            },
            1: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 2 },
        };
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: 0,
            controlsY: [startY + classicalRegHeight],
            targetsY: [startY, startY + classicalRegHeight * 2],
            label: '',
            width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
            children: [
                [
                    {
                        type: GateType.X,
                        x: startX + minGateWidth / 2,
                        controlsY: [],
                        targetsY: [startY],
                        label: 'X',
                        width: minGateWidth,
                    },
                ],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2,
                        controlsY: [],
                        targetsY: [[startY + classicalRegHeight * 2]],
                        label: 'H',
                        width: minGateWidth,
                    },
                ],
            ],
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('grouped gates', () => {
        const op: Operation = {
            gate: 'Foo',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            conditionalRender: ConditionalRender.AsGroup,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
            children: [
                {
                    gate: 'X',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                    conditionalRender: ConditionalRender.OnZero,
                },
                {
                    gate: 'H',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 1 }],
                    conditionalRender: ConditionalRender.OnOne,
                },
            ],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.Group,
            x: 0,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth + groupBoxPadding * 2,
            dataAttributes: { expanded: 'true' },
            children: [
                {
                    type: GateType.X,
                    x: startX + minGateWidth / 2,
                    controlsY: [],
                    targetsY: [startY],
                    label: 'X',
                    width: minGateWidth,
                },
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2,
                    controlsY: [],
                    targetsY: [[startY + registerHeight]],
                    label: 'H',
                    width: minGateWidth,
                },
            ],
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
    test('no render on null', () => {
        const metadata: Metadata = {
            type: GateType.Invalid,
            x: 0,
            controlsY: [],
            targetsY: [],
            label: '',
            width: -1,
        };
        expect(_opToMetadata(null, [])).toEqual(metadata);
    });
    test('Invalid register', () => {
        let op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 1 }],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
        };
        expect(() => _opToMetadata(op, registers)).toThrowError('ERROR: Qubit register with ID 1 not found.');

        op = {
            gate: 'X',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [{ type: RegisterType.Classical, qId: 0, cId: 2 }],
            targets: [],
        };
        expect(() => _opToMetadata(op, registers)).toThrowError('ERROR: No classical registers found for qubit ID 0.');
    });
    test('skipped registers', () => {
        const op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [{ type: RegisterType.Qubit, qId: 2 }],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            2: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.X,
            x: 0,
            controlsY: [],
            targetsY: [startY + registerHeight],
            label: 'X',
            width: minGateWidth,
        };
        expect(_opToMetadata(op, registers)).toEqual(metadata);
    });
});

describe('Testing _getRegY', () => {
    const registers: RegisterMap = {
        0: {
            type: RegisterType.Qubit,
            y: startY,
            children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
        },
    };
    test('quantum register', () => {
        const reg: Register = { type: RegisterType.Qubit, qId: 0 };
        expect(_getRegY(reg, registers)).toEqual(startY);
    });
    test('classical register', () => {
        const reg: Register = { type: RegisterType.Classical, qId: 0, cId: 0 };
        expect(_getRegY(reg, registers)).toEqual(startY + classicalRegHeight);
    });
    test('No children', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
        };
        const reg: Register = { type: RegisterType.Classical, qId: 0, cId: 0 };
        expect(() => _getRegY(reg, registers)).toThrowError('ERROR: No classical registers found for qubit ID 0.');
    });
    test('Null cId', () => {
        const reg: Register = { type: RegisterType.Classical, qId: 0 };
        expect(() => _getRegY(reg, registers)).toThrowError(
            'ERROR: No ID defined for classical register associated with qubit ID 0.',
        );
    });
    test('Invalid cId', () => {
        const reg: Register = { type: RegisterType.Classical, qId: 0, cId: 1 };
        expect(() => _getRegY(reg, registers)).toThrowError(
            'ERROR: Classical register ID 1 invalid for qubit ID 0 with 1 classical register(s).',
        );
    });
    test('Invalid register type', () => {
        const reg: Register = { type: 2, qId: 0, cId: 1 };
        expect(() => _getRegY(reg, registers)).toThrowError('ERROR: Unknown register type 2.');
    });
});

describe('Testing _splitTargetsY', () => {
    const registers: RegisterMap = {
        0: {
            type: RegisterType.Qubit,
            y: startY,
        },
        1: {
            type: RegisterType.Qubit,
            y: startY + registerHeight,
            children: [{ type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight }],
        },
        2: {
            type: RegisterType.Qubit,
            y: startY + registerHeight * 2 + classicalRegHeight,
            children: [
                { type: RegisterType.Classical, y: startY + registerHeight * 2 + classicalRegHeight * 2 },
                { type: RegisterType.Classical, y: startY + registerHeight * 2 + classicalRegHeight * 3 },
                { type: RegisterType.Classical, y: startY + registerHeight * 2 + classicalRegHeight * 4 },
            ],
        },
    };
    test('adjacent qubit regs', () => {
        const targets: Register[] = [
            { type: RegisterType.Qubit, qId: 0 },
            { type: RegisterType.Qubit, qId: 2 },
            { type: RegisterType.Qubit, qId: 1 },
        ];
        const targetsY: number[] = [startY, startY + registerHeight, startY + registerHeight * 2 + classicalRegHeight];
        expect(_splitTargetsY(targets, [], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [0], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [startY + registerHeight * 3], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [0, startY + registerHeight * 3], registers)).toEqual([targetsY]);
    });
    test('adjacent classical regs', () => {
        const targets: Register[] = [
            { type: RegisterType.Classical, qId: 2, cId: 0 },
            { type: RegisterType.Classical, qId: 2, cId: 2 },
            { type: RegisterType.Classical, qId: 2, cId: 1 },
        ];
        const targetsY: number[] = [
            startY + registerHeight * 2 + classicalRegHeight * 2,
            startY + registerHeight * 2 + classicalRegHeight * 3,
            startY + registerHeight * 2 + classicalRegHeight * 4,
        ];
        expect(_splitTargetsY(targets, [], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [0], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [startY + registerHeight * 3 + classicalRegHeight * 4], registers)).toEqual([
            targetsY,
        ]);
        expect(_splitTargetsY(targets, [0, startY + registerHeight * 3 + classicalRegHeight * 4], registers)).toEqual([
            targetsY,
        ]);
    });
    test('adjacent qubit/classical regs', () => {
        const targets: Register[] = [
            { type: RegisterType.Qubit, qId: 2 },
            { type: RegisterType.Classical, qId: 1, cId: 0 },
            { type: RegisterType.Classical, qId: 2, cId: 0 },
        ];
        const targetsY: number[] = [
            startY + registerHeight + classicalRegHeight,
            startY + registerHeight * 2 + classicalRegHeight,
            startY + registerHeight * 2 + classicalRegHeight * 2,
        ];
        expect(_splitTargetsY(targets, [], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [0], registers)).toEqual([targetsY]);
        expect(_splitTargetsY(targets, [startY + registerHeight * 3 + classicalRegHeight * 4], registers)).toEqual([
            targetsY,
        ]);
        expect(_splitTargetsY(targets, [0, startY + registerHeight * 3 + classicalRegHeight * 4], registers)).toEqual([
            targetsY,
        ]);
    });
    test('single target', () => {
        const targets: Register[] = [{ type: RegisterType.Qubit, qId: 0 }];
        expect(_splitTargetsY(targets, [], registers)).toEqual([[startY]]);
        expect(_splitTargetsY(targets, [0], registers)).toEqual([[startY]]);
        expect(_splitTargetsY(targets, [startY + registerHeight], registers)).toEqual([[startY]]);
        expect(_splitTargetsY(targets, [0, startY + registerHeight], registers)).toEqual([[startY]]);
    });
    test('split non-adjacent qubit regs', () => {
        const targets: Register[] = [
            { type: RegisterType.Qubit, qId: 0 },
            { type: RegisterType.Qubit, qId: 2 },
        ];
        const targetsY: number[][] = [[startY], [startY + registerHeight * 2 + classicalRegHeight]];
        expect(_splitTargetsY(targets, [], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [0], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [startY + registerHeight * 3], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [0, startY + registerHeight * 3], registers)).toEqual(targetsY);
    });
    test('split two qubit regs with classical register', () => {
        const targets: Register[] = [
            { type: RegisterType.Qubit, qId: 0 },
            { type: RegisterType.Qubit, qId: 1 },
        ];
        const targetsY: number[][] = [[startY], [startY + registerHeight]];
        expect(_splitTargetsY(targets, [startY + classicalRegHeight], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [0, startY + classicalRegHeight], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [startY + registerHeight * 2, startY + classicalRegHeight], registers)).toEqual(
            targetsY,
        );
        expect(
            _splitTargetsY(targets, [startY + registerHeight * 2, 0, startY + classicalRegHeight], registers),
        ).toEqual(targetsY);
    });
    test('split two classical regs with classical reg', () => {
        const targets: Register[] = [
            { type: RegisterType.Classical, qId: 2, cId: 0 },
            { type: RegisterType.Classical, qId: 2, cId: 2 },
        ];
        const targetsY: number[][] = [
            [startY + registerHeight * 2 + classicalRegHeight * 2],
            [startY + registerHeight * 2 + classicalRegHeight * 4],
        ];
        expect(_splitTargetsY(targets, [startY + registerHeight * 2 + classicalRegHeight * 3], registers)).toEqual(
            targetsY,
        );
        expect(_splitTargetsY(targets, [0, startY + registerHeight * 2 + classicalRegHeight * 3], registers)).toEqual(
            targetsY,
        );
        expect(
            _splitTargetsY(
                targets,
                [
                    startY + registerHeight * 3 + classicalRegHeight * 2,
                    startY + registerHeight * 2 + classicalRegHeight * 3,
                ],
                registers,
            ),
        ).toEqual(targetsY);
        expect(
            _splitTargetsY(
                targets,
                [
                    startY + registerHeight * 3 + classicalRegHeight * 2,
                    0,
                    startY + registerHeight * 2 + classicalRegHeight * 3,
                ],
                registers,
            ),
        ).toEqual(targetsY);
    });
    test('split multiple targets with classical register', () => {
        const targets: Register[] = [
            { type: RegisterType.Qubit, qId: 0 },
            { type: RegisterType.Qubit, qId: 2 },
            { type: RegisterType.Qubit, qId: 1 },
        ];
        let targetsY: number[][] = [
            [startY],
            [startY + registerHeight, startY + registerHeight * 2 + classicalRegHeight],
        ];
        expect(_splitTargetsY(targets, [startY + classicalRegHeight], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [10, startY + classicalRegHeight], registers)).toEqual(targetsY);
        expect(_splitTargetsY(targets, [60, startY + classicalRegHeight], registers)).toEqual(targetsY);

        targetsY = [[startY, startY + registerHeight], [startY + registerHeight * 2 + classicalRegHeight]];
        expect(_splitTargetsY(targets, [startY + registerHeight + classicalRegHeight], registers)).toEqual(targetsY);
    });
});

describe('Testing _offsetChildrenX', () => {
    const offset = 50;
    test('no grandchildren', () => {
        const children: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: 0,
                    controlsY: [],
                    targetsY: [],
                    width: minGateWidth,
                    label: 'X',
                },
            ],
        ];
        const expected: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: 50,
                    controlsY: [],
                    targetsY: [],
                    width: minGateWidth,
                    label: 'X',
                },
            ],
        ];
        _offsetChildrenX(children, offset);
        expect(children).toEqual(expected);
    });
    test('has grandchildren', () => {
        const children: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: 0,
                    controlsY: [],
                    targetsY: [],
                    width: minGateWidth,
                    label: 'X',
                    children: [
                        [
                            {
                                type: GateType.X,
                                x: 0,
                                controlsY: [],
                                targetsY: [],
                                width: minGateWidth,
                                label: 'X',
                            },
                        ],
                        [],
                    ],
                },
            ],
        ];
        const expected: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: 50,
                    controlsY: [],
                    targetsY: [],
                    width: minGateWidth,
                    label: 'X',
                    children: [
                        [
                            {
                                type: GateType.X,
                                x: 50,
                                controlsY: [],
                                targetsY: [],
                                width: minGateWidth,
                                label: 'X',
                            },
                        ],
                        [],
                    ],
                },
            ],
        ];
        _offsetChildrenX(children, offset);
        expect(children).toEqual(expected);
    });
    test('undefined child', () => {
        expect(() => _offsetChildrenX(undefined, offset)).not.toThrow();
    });
});

describe('Testing _fillMetadataX', () => {
    test('Non-classically-isControlled gate', () => {
        const columnWidths: number[] = Array(1).fill(minGateWidth);
        const expectedEndX = startX + minGateWidth + gatePadding * 2;
        const opsMetadata: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: 0,
                    controlsY: [],
                    targetsY: [],
                    label: 'X',
                    width: minGateWidth,
                },
            ],
        ];
        const expected: Metadata[][] = [
            [
                {
                    type: GateType.X,
                    x: startX + minGateWidth / 2,
                    controlsY: [],
                    targetsY: [],
                    label: 'X',
                    width: minGateWidth,
                },
            ],
        ];
        const endX: number = _fillMetadataX(opsMetadata, columnWidths);
        expect(opsMetadata).toEqual(expected);
        expect(endX).toEqual(expectedEndX);
    });
    test('classically-isControlled gate with no children', () => {
        const columnWidths: number[] = Array(1).fill(minGateWidth);
        const expectedEndX = startX + minGateWidth + gatePadding * 2;
        const opsMetadata: Metadata[][] = [
            [
                {
                    type: GateType.ClassicalControlled,
                    x: 0,
                    controlsY: [],
                    targetsY: [],
                    label: 'X',
                    width: minGateWidth,
                },
            ],
        ];
        const expected: Metadata[][] = [
            [
                {
                    type: GateType.ClassicalControlled,
                    x: startX,
                    controlsY: [],
                    targetsY: [],
                    label: 'X',
                    width: minGateWidth,
                },
            ],
        ];
        const endX: number = _fillMetadataX(opsMetadata, columnWidths);
        expect(opsMetadata).toEqual(expected);
        expect(endX).toEqual(expectedEndX);
    });
    test('depth-1 children', () => {
        const columnWidths: number[] = Array(1).fill(minGateWidth + gatePadding * 2);
        const expectedEndX = startX + minGateWidth + gatePadding * 4;
        const opsMetadata: Metadata[][] = [
            [
                {
                    type: GateType.ClassicalControlled,
                    x: 0,
                    controlsY: [],
                    targetsY: [],
                    children: [
                        [
                            {
                                type: GateType.X,
                                x: 0,
                                controlsY: [],
                                targetsY: [],
                                label: 'X',
                                width: minGateWidth,
                            },
                        ],
                        [
                            {
                                type: GateType.X,
                                x: 0,
                                controlsY: [],
                                targetsY: [],
                                label: 'X',
                                width: minGateWidth,
                            },
                        ],
                    ],
                    label: 'X',
                    width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
                },
            ],
        ];
        const expected: Metadata[][] = [
            [
                {
                    type: GateType.ClassicalControlled,
                    x: startX,
                    controlsY: [],
                    targetsY: [],
                    children: [
                        [
                            {
                                type: GateType.X,
                                x: controlBtnOffset + groupBoxPadding,
                                controlsY: [],
                                targetsY: [],
                                label: 'X',
                                width: minGateWidth,
                            },
                        ],
                        [
                            {
                                type: GateType.X,
                                x: controlBtnOffset + groupBoxPadding,
                                controlsY: [],
                                targetsY: [],
                                label: 'X',
                                width: minGateWidth,
                            },
                        ],
                    ],
                    label: 'X',
                    width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
                },
            ],
        ];

        const endX: number = _fillMetadataX(opsMetadata, columnWidths);
        expect(opsMetadata).toEqual(expected);
        expect(endX).toEqual(expectedEndX);
    });
    test('empty args', () => {
        const opsMetadata: Metadata[][] = [];
        const endX: number = _fillMetadataX(opsMetadata, []);
        expect(opsMetadata).toEqual([]);
        expect(endX).toEqual(startX);
    });
});

describe('Testing processOperations', () => {
    test('single qubit gates', () => {
        const rxWidth = 52;
        const operations: Operation[] = [
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ qId: 1 }],
            },
            {
                gate: 'RX',
                displayArgs: '(0.25)',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ qId: 1 }],
            },
        ];
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const expectedOps: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + (minGateWidth + gatePadding * 2) + rxWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY + registerHeight]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + (minGateWidth + gatePadding * 2) + rxWidth / 2,
                controlsY: [],
                targetsY: [[startY + registerHeight]],
                label: 'RX',
                displayArgs: '(0.25)',
                width: rxWidth,
            },
        ];
        const expectedWidth: number = startX + minGateWidth + rxWidth + gatePadding * 4;
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual(expectedOps);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('single wide qubit gates', () => {
        const expectedCustomWidth = 67;
        const operations: Operation[] = [
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'FooBar',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const expectedOps: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + (minGateWidth + gatePadding * 2) + expectedCustomWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'FooBar',
                width: expectedCustomWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY + registerHeight]],
                label: 'H',
                width: minGateWidth,
            },
        ];
        const expectedWidth: number = startX + minGateWidth + expectedCustomWidth + gatePadding * 4;
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual(expectedOps);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('single and multi qubit gates', () => {
        const operations: Operation[] = [
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'X',
                isMeasurement: false,
                isConditional: false,
                isControlled: true,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 1 }],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
        ];
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const expectedOps: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Cnot,
                x: startX + (minGateWidth + gatePadding * 2) + minGateWidth / 2,
                controlsY: [startY + registerHeight],
                targetsY: [startY],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + (minGateWidth + gatePadding * 2) * 2 + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY + registerHeight]],
                label: 'H',
                width: minGateWidth,
            },
        ];
        const expectedWidth: number = startX + (minGateWidth + gatePadding * 2) * 3;
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual(expectedOps);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('classically-controlled gates', () => {
        const op: Operation = {
            gate: 'X',
            isMeasurement: false,
            isConditional: true,
            isControlled: true,
            isAdjoint: false,
            controls: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
            children: [
                {
                    gate: 'X',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                    conditionalRender: ConditionalRender.OnZero,
                },
                {
                    gate: 'H',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 1 }],
                    conditionalRender: ConditionalRender.OnOne,
                },
                {
                    gate: 'Z',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                    conditionalRender: ConditionalRender.OnZero,
                },
            ],
        };
        const registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
            },
            1: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 2 },
        };
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + classicalRegHeight],
            targetsY: [startY, startY + classicalRegHeight * 2],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + controlBtnOffset + groupBoxPadding * 2,
            children: [
                [
                    {
                        type: GateType.X,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        controlsY: [],
                        targetsY: [startY],
                        label: 'X',
                        width: minGateWidth,
                    },
                    {
                        type: GateType.Unitary,
                        x:
                            startX +
                            minGateWidth +
                            minGateWidth / 2 +
                            gatePadding * 2 +
                            controlBtnOffset +
                            groupBoxPadding,
                        controlsY: [],
                        targetsY: [[startY]],
                        label: 'Z',
                        width: minGateWidth,
                    },
                ],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        controlsY: [],
                        targetsY: [[startY + classicalRegHeight * 2]],
                        label: 'H',
                        width: minGateWidth,
                    },
                ],
            ],
        };
        const expectedWidth: number =
            startX + minGateWidth * 2 + gatePadding * 4 + controlBtnOffset + groupBoxPadding * 2;
        const { metadataList, svgWidth } = processOperations([op], registers);
        expect(metadataList).toEqual([metadata]);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('grouped gates', () => {
        const op: Operation = {
            gate: 'Foo',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
            conditionalRender: ConditionalRender.AsGroup,
            children: [
                {
                    gate: 'X',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                },
                {
                    gate: 'H',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 1 }],
                },
                {
                    gate: 'Z',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                },
            ],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 2,
            dataAttributes: { expanded: 'true' },
            children: [
                {
                    type: GateType.X,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [startY],
                    label: 'X',
                    width: minGateWidth,
                },
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth + minGateWidth / 2 + gatePadding * 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
                    label: 'Z',
                    width: minGateWidth,
                },
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY + registerHeight]],
                    label: 'H',
                    width: minGateWidth,
                },
            ],
        };
        const expectedWidth: number = startX + minGateWidth * 2 + gatePadding * 4 + groupBoxPadding * 2;
        const { metadataList, svgWidth } = processOperations([op], registers);
        expect(metadataList).toEqual([metadata]);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('nested grouped gates', () => {
        const op: Operation = {
            gate: 'Foo',
            isMeasurement: false,
            isConditional: false,
            isControlled: false,
            isAdjoint: false,
            controls: [],
            targets: [
                { type: RegisterType.Qubit, qId: 0 },
                { type: RegisterType.Qubit, qId: 1 },
            ],
            conditionalRender: ConditionalRender.AsGroup,
            children: [
                {
                    gate: 'Foo',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 0 }],
                    conditionalRender: ConditionalRender.AsGroup,
                    children: [
                        {
                            gate: 'X',
                            isMeasurement: false,
                            isConditional: false,
                            isControlled: false,
                            isAdjoint: false,
                            controls: [],
                            targets: [{ type: RegisterType.Qubit, qId: 0 }],
                        },
                        {
                            gate: 'Z',
                            isMeasurement: false,
                            isConditional: false,
                            isControlled: false,
                            isAdjoint: false,
                            controls: [],
                            targets: [{ type: RegisterType.Qubit, qId: 0 }],
                        },
                    ],
                },
                {
                    gate: 'H',
                    isMeasurement: false,
                    isConditional: false,
                    isControlled: false,
                    isAdjoint: false,
                    controls: [],
                    targets: [{ type: RegisterType.Qubit, qId: 1 }],
                },
            ],
        };
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 4,
            dataAttributes: { expanded: 'true' },
            children: [
                {
                    type: GateType.Group,
                    x: startX + gatePadding,
                    controlsY: [],
                    targetsY: [startY],
                    label: '',
                    width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 2,
                    dataAttributes: { expanded: 'true' },
                    children: [
                        {
                            type: GateType.X,
                            x: startX + minGateWidth / 2 + groupBoxPadding * 2,
                            controlsY: [],
                            targetsY: [startY],
                            label: 'X',
                            width: minGateWidth,
                        },
                        {
                            type: GateType.Unitary,
                            x: startX + minGateWidth + minGateWidth / 2 + gatePadding * 2 + groupBoxPadding * 2,
                            controlsY: [],
                            targetsY: [[startY]],
                            label: 'Z',
                            width: minGateWidth,
                        },
                    ],
                },
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth + gatePadding + groupBoxPadding * 2, // startX + half of above gate's width + padding
                    controlsY: [],
                    targetsY: [[startY + registerHeight]],
                    label: 'H',
                    width: minGateWidth,
                },
            ],
        };
        const expectedWidth: number = startX + minGateWidth * 2 + gatePadding * 4 + groupBoxPadding * 4;
        const { metadataList, svgWidth } = processOperations([op], registers);
        expect(metadataList).toEqual([metadata]);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('measure gates', () => {
        const operations: Operation[] = [
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Classical, qId: 0, cId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 1 }],
            },
            {
                gate: 'M',
                isMeasurement: true,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [{ type: RegisterType.Qubit, qId: 0 }],
                targets: [{ type: RegisterType.Classical, qId: 0, cId: 1 }],
            },
        ];
        const registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [
                    { type: RegisterType.Classical, y: startY + classicalRegHeight },
                    { type: RegisterType.Classical, y: startY + classicalRegHeight * 2 },
                ],
            },
            1: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 3 },
        };
        const expectedOps: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Measure,
                x: startX + minGateWidth + gatePadding * 2 + minGateWidth / 2,
                controlsY: [startY],
                targetsY: [startY + classicalRegHeight],
                label: '',
                width: minGateWidth,
            },
            {
                type: GateType.Measure,
                x: startX + (minGateWidth + gatePadding * 2) * 2 + minGateWidth / 2,
                controlsY: [startY],
                targetsY: [startY + classicalRegHeight * 2],
                label: '',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY + classicalRegHeight * 3]],
                label: 'H',
                width: minGateWidth,
            },
        ];
        const expectedWidth: number = startX + (minGateWidth + gatePadding * 2) * 3;
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual(expectedOps);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('skipped registers', () => {
        const operations: Operation[] = [
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 0 }],
            },
            {
                gate: 'H',
                isMeasurement: false,
                isConditional: false,
                isControlled: false,
                isAdjoint: false,
                controls: [],
                targets: [{ type: RegisterType.Qubit, qId: 2 }],
            },
        ];
        const registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [
                    { type: RegisterType.Classical, y: startY + classicalRegHeight },
                    { type: RegisterType.Classical, y: startY + classicalRegHeight * 2 },
                ],
            },
            2: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 3 },
        };
        const expectedOps: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY]],
                label: 'H',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2,
                controlsY: [],
                targetsY: [[startY + classicalRegHeight * 3]],
                label: 'H',
                width: minGateWidth,
            },
        ];
        const expectedWidth: number = startX + minGateWidth + gatePadding * 2;
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual(expectedOps);
        expect(svgWidth).toEqual(expectedWidth);
    });
    test('no operations', () => {
        const operations: Operation[] = [];
        const registers: RegisterMap = {};
        const { metadataList, svgWidth } = processOperations(operations, registers);
        expect(metadataList).toEqual([]);
        expect(svgWidth).toEqual(startX);
    });
});
