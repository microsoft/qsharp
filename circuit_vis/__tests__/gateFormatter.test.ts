import {
    formatGates,
    _formatGate,
    _createGate,
    _measure,
    _unitary,
    _swap,
    _controlledGate,
    _groupedOperations,
    _classicalControlled,
    _zoomButton,
} from '../src/formatters/gateFormatter';
import { createSvgElement } from '../src/formatters/formatUtils';
import { Metadata, GateType } from '../src/metadata';
import {
    startX,
    startY,
    registerHeight,
    minGateWidth,
    gatePadding,
    classicalRegHeight,
    controlBtnOffset,
    groupBoxPadding,
} from '../src/constants';

describe('Testing _classicalControlled', () => {
    test("one 'zero' child", () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + classicalRegHeight],
            targetsY: [startY],
            width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
            label: '',
            children: [
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        label: 'X',
                        controlsY: [],
                        targetsY: [[startY]],
                        width: minGateWidth,
                    },
                ],
                [],
            ],
        };
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test("one 'one' child", () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + classicalRegHeight],
            targetsY: [startY],
            width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
            label: '',
            children: [
                [],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        label: 'X',
                        controlsY: [],
                        targetsY: [[startY]],
                        width: minGateWidth,
                    },
                ],
            ],
        };
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test("one 'zero'/'one' child", () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + classicalRegHeight],
            targetsY: [startY, startY + classicalRegHeight * 2],
            label: '',
            width: minGateWidth + controlBtnOffset + groupBoxPadding * 2,
            children: [
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        controlsY: [],
                        targetsY: [[startY]],
                        label: 'X',
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
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test("multiple 'zero'/'one' children", () => {
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
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + controlBtnOffset + groupBoxPadding,
                        controlsY: [],
                        targetsY: [[startY]],
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
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test('nested children', () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + registerHeight * 2],
            targetsY: [startY, startY + registerHeight],
            width: minGateWidth * 2 + gatePadding * 6,
            label: 'if',
            children: [
                [],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + gatePadding,
                        label: 'X',
                        controlsY: [],
                        targetsY: [[startY]],
                        width: minGateWidth,
                    },
                    {
                        type: GateType.ClassicalControlled,
                        x: startX + minGateWidth + gatePadding * 3,
                        controlsY: [startY + registerHeight * 3],
                        targetsY: [startY, startY + registerHeight],
                        width: minGateWidth + gatePadding * 2,
                        label: 'if',
                        children: [
                            [],
                            [
                                {
                                    type: GateType.Cnot,
                                    x: startX + minGateWidth + gatePadding * 4 + minGateWidth / 2,
                                    label: 'X',
                                    controlsY: [startY + registerHeight],
                                    targetsY: [startY],
                                    width: minGateWidth,
                                },
                            ],
                        ],
                    },
                ],
            ],
        };
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test('No htmlClass', () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + registerHeight * 2],
            targetsY: [startY, startY + registerHeight],
            width: minGateWidth * 2 + gatePadding * 4,
            label: 'if',
            children: [
                [],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + gatePadding,
                        label: 'X',
                        controlsY: [],
                        targetsY: [[startY]],
                        width: minGateWidth,
                    },
                ],
            ],
        };
        expect(_classicalControlled(metadata)).toMatchSnapshot();
    });
    test('change padding', () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + registerHeight * 2],
            targetsY: [startY, startY + registerHeight],
            width: minGateWidth * 2 + gatePadding * 4,
            label: 'if',
            children: [
                [],
                [
                    {
                        type: GateType.Unitary,
                        x: startX + minGateWidth / 2 + gatePadding,
                        label: 'X',
                        controlsY: [],
                        targetsY: [[startY]],
                        width: minGateWidth,
                    },
                ],
            ],
        };
        expect(_classicalControlled(metadata, 20)).toMatchSnapshot();
    });
});

describe('Testing _groupedOperations', () => {
    test('one child', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY],
            label: '',
            width: minGateWidth + groupBoxPadding * 2,
            children: [
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
                    label: 'X',
                    width: minGateWidth,
                },
            ],
        };
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
    test('children on consecutive registers', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth + groupBoxPadding * 2,
            children: [
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
                    label: 'X',
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
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
    test('children on non-consecutive registers', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight * 2],
            label: '',
            width: minGateWidth + groupBoxPadding * 2,
            children: [
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
                    label: 'X',
                    width: minGateWidth,
                },
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY + registerHeight * 2]],
                    label: 'H',
                    width: minGateWidth,
                },
            ],
        };
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
    test('children on same register', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 2,
            children: [
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
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
            ],
        };
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
    test('multiple children', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 2,
            children: [
                {
                    type: GateType.Unitary,
                    x: startX + minGateWidth / 2 + groupBoxPadding,
                    controlsY: [],
                    targetsY: [[startY]],
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
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
    test('nested children', () => {
        const metadata: Metadata = {
            type: GateType.Group,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 4,
            children: [
                {
                    type: GateType.Group,
                    x: startX + gatePadding,
                    controlsY: [],
                    targetsY: [startY],
                    label: '',
                    width: minGateWidth * 2 + gatePadding * 2 + groupBoxPadding * 2,
                    children: [
                        {
                            type: GateType.Unitary,
                            x: startX + minGateWidth / 2 + groupBoxPadding * 2,
                            controlsY: [],
                            targetsY: [[startY]],
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
                    x: startX + minGateWidth + gatePadding + groupBoxPadding * 2,
                    controlsY: [],
                    targetsY: [[startY + registerHeight]],
                    label: 'H',
                    width: minGateWidth,
                },
            ],
        };
        expect(_groupedOperations(metadata, 0)).toMatchSnapshot();
    });
});

describe('Testing _controlledGate', () => {
    test('CNOT gate', () => {
        const metadata: Metadata = {
            type: GateType.Cnot,
            label: 'X',
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight],
            width: minGateWidth,
        };
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Flip target and control
        metadata.controlsY = [startY + registerHeight];
        metadata.targetsY = [startY];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('SWAP gate', () => {
        const metadata: Metadata = {
            type: GateType.Swap,
            label: '',
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight, startY + registerHeight * 2],
            width: minGateWidth,
        };
        // Control on top
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Control on bottom
        metadata.controlsY = [startY + registerHeight * 2];
        metadata.targetsY = [startY, startY + registerHeight];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Control in middle
        metadata.controlsY = [startY + registerHeight];
        metadata.targetsY = [startY, startY + registerHeight * 2];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Controlled U gate with 1 control + 1 target', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            label: 'Foo',
            x: startX,
            controlsY: [startY],
            targetsY: [[startY + registerHeight]],
            width: 45,
        };
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Flip target and control
        metadata.controlsY = [startY + registerHeight];
        metadata.targetsY = [[startY]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Controlled U gate with multiple controls + 1 target', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            label: 'Foo',
            x: startX,
            controlsY: [startY, startY + registerHeight],
            targetsY: [[startY + registerHeight * 2]],
            width: 45,
        };
        // Target on bottom
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Target on top
        metadata.controlsY = [startY + registerHeight, startY + registerHeight * 2];
        metadata.targetsY = [[startY]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Target in middle
        metadata.controlsY = [startY, startY + registerHeight * 2];
        metadata.targetsY = [[startY + registerHeight]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Controlled U gate with 1 control + 2 targets', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            label: 'Foo',
            x: startX,
            controlsY: [startY + registerHeight * 2],
            targetsY: [[startY, startY + registerHeight]],
            width: 45,
        };
        // Control on bottom
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Control on top
        metadata.controlsY = [startY];
        metadata.targetsY = [[startY + registerHeight, startY + registerHeight * 2]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Control in middle
        metadata.controlsY = [startY + registerHeight];
        metadata.targetsY = [[startY], [startY + registerHeight * 2]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Controlled U gate with 2 controls + 2 targets', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            label: 'Foo',
            x: startX,
            controlsY: [startY + registerHeight * 2, startY + registerHeight * 3],
            targetsY: [[startY, startY + registerHeight]],
            width: 45,
        };
        // Controls on bottom
        let svg: SVGElement = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Controls on top
        metadata.controlsY = [startY, startY + registerHeight];
        metadata.targetsY = [[startY + registerHeight * 2, startY + registerHeight * 3]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Controls in middle
        metadata.controlsY = [startY + registerHeight, startY + registerHeight * 2];
        metadata.targetsY = [[startY], [startY + registerHeight * 3]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();

        // Interleaved controls/targets
        metadata.controlsY = [startY + registerHeight, startY + registerHeight * 3];
        metadata.targetsY = [[startY], [startY + registerHeight * 2]];
        svg = _controlledGate(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Invalid gate', () => {
        const metadata: Metadata = {
            type: GateType.Measure,
            label: 'X',
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight],
            width: minGateWidth,
        };
        expect(() => _controlledGate(metadata, 0)).toThrowError(
            `ERROR: Unrecognized gate: X of type ${GateType.Measure}`,
        );
    });
});

describe('Testing _swap', () => {
    const metadata: Metadata = {
        type: GateType.Measure,
        label: 'SWAP',
        x: startX,
        controlsY: [],
        targetsY: [startY + registerHeight],
        width: minGateWidth,
    };

    test('Adjacent swap', () => {
        metadata.targetsY = [startY, startY + registerHeight];
        let svg: SVGElement = _swap(metadata, 0);
        expect(svg).toMatchSnapshot();
        // Flip target and control
        metadata.targetsY = [startY + registerHeight, startY];
        svg = _swap(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
    test('Non-adjacent swap', () => {
        metadata.targetsY = [startY, startY + registerHeight * 2];
        let svg: SVGElement = _swap(metadata, 0);
        expect(svg).toMatchSnapshot();
        // Flip target and control
        metadata.targetsY = [startY + registerHeight * 2, startY];
        svg = _swap(metadata, 0);
        expect(svg).toMatchSnapshot();
    });
});

describe('Testing _unitary', () => {
    test('Single qubit unitary', () => {
        expect(_unitary('H', startX, [[startY]], minGateWidth)).toMatchSnapshot();
    });
    test('Multiqubit unitary on consecutive registers', () => {
        let svg: SVGElement = _unitary('ZZ', startX, [[startY, startY + registerHeight]], minGateWidth);
        expect(svg).toMatchSnapshot();
        svg = _unitary('ZZZ', startX, [[startY, startY + registerHeight, startY + registerHeight * 2]], minGateWidth);
        expect(svg).toMatchSnapshot();
    });
    test('Multiqubit unitary on non-consecutive registers', () => {
        // Dashed line between unitaries
        let svg: SVGElement = _unitary('ZZ', startX, [[startY], [startY + registerHeight * 2]], minGateWidth);
        expect(svg).toMatchSnapshot();
        svg = _unitary(
            'ZZZ',
            startX,
            [[startY], [startY + registerHeight * 2, startY + registerHeight * 3]],
            minGateWidth,
        );
        expect(svg).toMatchSnapshot();
        // Solid line
        svg = _unitary('ZZ', startX, [[startY], [startY + registerHeight * 2]], minGateWidth, undefined, false);
        expect(svg).toMatchSnapshot();
        svg = _unitary(
            'ZZZ',
            startX,
            [[startY], [startY + registerHeight * 2, startY + registerHeight * 3]],
            minGateWidth,
            undefined,
            false,
        );
        expect(svg).toMatchSnapshot();
    });
    test('No y coords', () => {
        expect(() => _unitary('ZZ', startX, [], minGateWidth)).toThrowError(
            'Failed to render unitary gate (ZZ): has no y-values',
        );
    });
});

describe('Testing _measure', () => {
    test('1 qubit + 1 classical registers', () => {
        expect(_measure(startX, startY)).toMatchSnapshot();
    });
    test('2 qubit + 1 classical registers', () => {
        expect(_measure(startX, startY)).toMatchSnapshot();
    });
    test('2 qubit + 2 classical registers', () => {
        expect(_measure(startX, startY)).toMatchSnapshot();
        expect(_measure(startX, startY + registerHeight)).toMatchSnapshot();
    });
});

describe('Testing _createGate', () => {
    const metadata: Metadata = {
        type: GateType.Invalid,
        x: 0,
        controlsY: [],
        targetsY: [],
        label: '',
        width: -1,
        dataAttributes: { a: '1', b: '2' },
    };
    const line: SVGElement = createSvgElement('line');
    test('Empty gate', () => {
        expect(_createGate([line], metadata, 0).outerHTML).toEqual(
            '<g class="gate" data-a="1" data-b="2"><line></line></g>',
        );
    });
    test('Expanded gate', () => {
        if (metadata.dataAttributes) metadata.dataAttributes['expanded'] = 'true';
        expect(_createGate([line], metadata, 0)).toMatchSnapshot();
    });
});

describe('Testing _zoomButton', () => {
    const metadata: Metadata = {
        type: GateType.Group,
        x: startX,
        controlsY: [],
        targetsY: [startY],
        label: '',
        width: minGateWidth + groupBoxPadding * 2,
        children: [
            {
                type: GateType.Unitary,
                x: startX + minGateWidth / 2 + groupBoxPadding,
                controlsY: [],
                targetsY: [[startY]],
                label: 'X',
                width: minGateWidth,
            },
        ],
    };

    test('Expanded gate', () => {
        if (metadata.dataAttributes) {
            metadata.dataAttributes['expanded'] = 'true';
            metadata.dataAttributes['zoom-in'] = 'true';
        }
        expect(_zoomButton(metadata, 0)).toMatchSnapshot();
    });
    test('Non-expanded with no children gate', () => {
        if (metadata.dataAttributes) {
            delete metadata.dataAttributes['expanded'];
            delete metadata.dataAttributes['zoom-in'];
        }
        expect(_zoomButton(metadata, 0)).toMatchSnapshot();
    });
    test('Non-expanded with children gate', () => {
        if (metadata.dataAttributes) {
            delete metadata.dataAttributes['expanded'];
            metadata.dataAttributes['zoom-in'] = 'true';
        }
        expect(_zoomButton(metadata, 0)).toMatchSnapshot();
    });
    test('Expanded with children gate', () => {
        if (metadata.dataAttributes) {
            metadata.dataAttributes['expanded'] = 'true';
            metadata.dataAttributes['zoom-in'] = 'true';
        }
        expect(_zoomButton(metadata, 0)).toMatchSnapshot();
    });
});

describe('Testing _formatGate', () => {
    test('measure gate', () => {
        const metadata: Metadata = {
            type: GateType.Measure,
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight],
            label: '',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('single-qubit unitary gate', () => {
        const metadata: Metadata = {
            type: GateType.Unitary,
            x: startX,
            controlsY: [],
            targetsY: [[startY]],
            label: 'H',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('single-qubit unitary gate with arguments', () => {
        const metadata: Metadata = {
            type: GateType.Unitary,
            x: startX,
            controlsY: [],
            targetsY: [[startY]],
            label: 'Ry',
            displayArgs: '(0.25)',
            width: 52,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('multi-qubit unitary gate', () => {
        const metadata: Metadata = {
            type: GateType.Unitary,
            x: startX,
            controlsY: [],
            targetsY: [[startY, startY + registerHeight]],
            label: 'U',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('multi-qubit unitary gate with arguments', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            x: startX,
            controlsY: [],
            targetsY: [[startY, startY + registerHeight]],
            label: 'U',
            displayArgs: "('foo', 'bar')",
            width: 77,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('swap gate', () => {
        const metadata: Metadata = {
            type: GateType.Swap,
            x: startX,
            controlsY: [],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('controlled swap gate', () => {
        const metadata: Metadata = {
            type: GateType.Swap,
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight, startY + registerHeight * 2],
            label: '',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('CNOT gate', () => {
        const metadata: Metadata = {
            type: GateType.Cnot,
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight],
            label: 'X',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('controlled unitary gate', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            x: startX,
            controlsY: [startY],
            targetsY: [[startY + registerHeight]],
            label: 'U',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('controlled unitary gate with arguments', () => {
        const metadata: Metadata = {
            type: GateType.ControlledUnitary,
            x: startX,
            controlsY: [startY],
            targetsY: [[startY + registerHeight]],
            label: 'U',
            displayArgs: "('foo', 'bar')",
            width: 77,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('classically controlled gate', () => {
        const metadata: Metadata = {
            type: GateType.ClassicalControlled,
            x: startX,
            controlsY: [startY + registerHeight * 2],
            targetsY: [startY, startY + registerHeight],
            label: '',
            width: minGateWidth,
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('gate with metadata', () => {
        const metadata: Metadata = {
            type: GateType.Unitary,
            x: startX,
            controlsY: [],
            targetsY: [[startY]],
            label: 'H',
            width: minGateWidth,
            dataAttributes: { a: '1', b: '2' },
        };
        expect(_formatGate(metadata)).toMatchSnapshot();
    });
    test('invalid gate', () => {
        const metadata: Metadata = {
            type: GateType.Invalid,
            x: startX,
            controlsY: [startY],
            targetsY: [startY + registerHeight],
            label: 'Foo',
            width: 48,
        };
        expect(() => _formatGate(metadata)).toThrowError(`ERROR: unknown gate (Foo) of type ${GateType.Invalid}.`);
    });
});

describe('Testing formatGates', () => {
    test('Single gate', () => {
        const gates: Metadata[] = [
            {
                type: GateType.Cnot,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + registerHeight],
                label: 'X',
                width: minGateWidth,
            },
        ];
        expect(formatGates(gates)).toMatchSnapshot();
    });
    test('Single null gate', () => {
        const gates: Metadata[] = [
            {
                type: GateType.Invalid,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + registerHeight],
                label: '',
                width: minGateWidth,
            },
        ];
        expect(() => formatGates(gates)).toThrowError(`ERROR: unknown gate () of type ${GateType.Invalid}.`);
    });
    test('Multiple gates', () => {
        const gates: Metadata[] = [
            {
                type: GateType.Cnot,
                x: startX,
                controlsY: [startY + registerHeight],
                targetsY: [startY],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.ControlledUnitary,
                x: startX,
                controlsY: [startY + registerHeight],
                targetsY: [[startY + registerHeight * 2]],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.Unitary,
                x: startX,
                controlsY: [],
                targetsY: [[startY + registerHeight * 2]],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.Measure,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + registerHeight * 3],
                label: 'X',
                width: minGateWidth,
            },
        ];
        expect(formatGates(gates)).toMatchSnapshot();
    });
    test('Multiple gates with invalid gate', () => {
        const gates: Metadata[] = [
            {
                type: GateType.Unitary,
                x: startX,
                controlsY: [],
                targetsY: [[startY + registerHeight * 2]],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.Cnot,
                x: startX,
                controlsY: [startY + registerHeight],
                targetsY: [startY],
                label: 'X',
                width: minGateWidth,
            },
            {
                type: GateType.Invalid,
                x: startX,
                controlsY: [],
                targetsY: [startY + registerHeight * 2],
                label: '',
                width: minGateWidth,
            },
            {
                type: GateType.Invalid,
                x: startX,
                controlsY: [],
                targetsY: [],
                label: '',
                width: minGateWidth,
            },
        ];
        expect(() => formatGates(gates)).toThrowError(`ERROR: unknown gate () of type ${GateType.Invalid}.`);
    });
});
