import { formatRegisters, _classicalRegister, _qubitRegister } from '../src/formatters/registerFormatter';
import { RegisterMap, RegisterType } from '../src/register';
import { Metadata, GateType } from '../src/metadata';
import { startY, registerHeight, classicalRegHeight, startX, minGateWidth } from '../src/constants';

describe('Testing _classicalRegister', () => {
    test('register with normal width', () => {
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
    });
    test('register with small width', () => {
        expect(_classicalRegister(10, 10, 5, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 5, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 5, 20)).toMatchSnapshot();
    });
    test('register with large width', () => {
        expect(_classicalRegister(10, 10, 500, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 500, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 500, 20)).toMatchSnapshot();
    });
    test('register with label offset', () => {
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
        expect(_classicalRegister(10, 10, 100, 20)).toMatchSnapshot();
    });
});

describe('Testing _qubitRegister', () => {
    test('register with normal width', () => {
        expect(_qubitRegister(0, 100, 20)).toMatchSnapshot();
        expect(_qubitRegister(1, 100, 20)).toMatchSnapshot();
        expect(_qubitRegister(2, 100, 20)).toMatchSnapshot();
    });
    test('register with small width', () => {
        expect(_qubitRegister(0, 5, 20)).toMatchSnapshot();
        expect(_qubitRegister(1, 5, 20)).toMatchSnapshot();
        expect(_qubitRegister(2, 5, 20)).toMatchSnapshot();
    });
    test('register with large width', () => {
        expect(_qubitRegister(0, 500, 20)).toMatchSnapshot();
        expect(_qubitRegister(1, 500, 20)).toMatchSnapshot();
        expect(_qubitRegister(2, 500, 20)).toMatchSnapshot();
    });
    test('register with label offset', () => {
        expect(_qubitRegister(0, 100, 20, 0)).toMatchSnapshot();
        expect(_qubitRegister(1, 100, 20, 5)).toMatchSnapshot();
        expect(_qubitRegister(2, 100, 20, 50)).toMatchSnapshot();
    });
});

describe('Testing formatRegisters', () => {
    test('1 quantum register', () => {
        const registers: RegisterMap = { 0: { type: RegisterType.Qubit, y: startY } };
        // Normal width
        expect(formatRegisters(registers, [], startX + 100)).toMatchSnapshot();
        // Small width
        expect(formatRegisters(registers, [], startX + 5)).toMatchSnapshot();
        // Large width
        expect(formatRegisters(registers, [], startX + 500)).toMatchSnapshot();
    });
    test('Multiple quantum registers', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
            3: { type: RegisterType.Qubit, y: startY + registerHeight * 3 },
        };
        // Normal width
        expect(formatRegisters(registers, [], startX + 100)).toMatchSnapshot();
        // Small width
        expect(formatRegisters(registers, [], startX + 5)).toMatchSnapshot();
        // Large width
        expect(formatRegisters(registers, [], startX + 500)).toMatchSnapshot();
    });
    test('Skipped quantum registers', () => {
        const registers: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
            3: { type: RegisterType.Qubit, y: startY + registerHeight * 3 },
        };
        // Normal width
        expect(formatRegisters(registers, [], startX + 100)).toMatchSnapshot();
        // Small width
        expect(formatRegisters(registers, [], startX + 5)).toMatchSnapshot();
        // Large width
        expect(formatRegisters(registers, [], startX + 500)).toMatchSnapshot();
    });
    test('Quantum and classical registers', () => {
        let registers: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
            },
            1: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 2 },
            2: { type: RegisterType.Qubit, y: startY + registerHeight + classicalRegHeight * 2 },
        };
        let measureGates: Metadata[] = [
            {
                type: GateType.Measure,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + classicalRegHeight],
                label: '',
                width: minGateWidth,
            },
        ];
        // Normal width
        expect(formatRegisters(registers, measureGates, startX + 100)).toMatchSnapshot();
        // Small width
        expect(formatRegisters(registers, measureGates, startX + 5)).toMatchSnapshot();
        // Large width
        expect(formatRegisters(registers, measureGates, startX + 500)).toMatchSnapshot();

        registers = {
            0: { type: RegisterType.Qubit, y: startY },
            1: {
                type: RegisterType.Qubit,
                y: startY + registerHeight,
                children: [
                    { type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight },
                    { type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight * 2 },
                ],
            },
            2: {
                type: RegisterType.Qubit,
                y: startY + registerHeight + classicalRegHeight * 3,
                children: [{ type: RegisterType.Classical, y: startY + registerHeight + classicalRegHeight * 4 }],
            },
        };
        measureGates = [
            {
                type: GateType.Measure,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + classicalRegHeight],
                label: '',
                width: minGateWidth,
            },
            {
                type: GateType.Measure,
                x: startX,
                controlsY: [startY],
                targetsY: [startY + classicalRegHeight * 2],
                label: '',
                width: minGateWidth,
            },
            {
                type: GateType.Measure,
                x: startX,
                controlsY: [startY + classicalRegHeight * 3],
                targetsY: [startY + classicalRegHeight * 4],
                label: '',
                width: minGateWidth,
            },
        ];
        // Normal width
        expect(formatRegisters(registers, measureGates, startX + 100)).toMatchSnapshot();
        // Small width
        expect(formatRegisters(registers, measureGates, startX + 5)).toMatchSnapshot();
        // Large width
        expect(formatRegisters(registers, measureGates, startX + 500)).toMatchSnapshot();
    });
});
