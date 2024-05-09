import { Qubit } from '../src/circuit';
import { RegisterMap, RegisterType } from '../src/register';
import { formatInputs, _qubitInput } from '../src/formatters/inputFormatter';
import { startY, registerHeight, classicalRegHeight } from '../src/constants';

describe('Testing _qubitInput', () => {
    test('classical register', () => {
        expect(_qubitInput(20)).toMatchSnapshot();
        expect(_qubitInput(50)).toMatchSnapshot();
        expect(_qubitInput(0)).toMatchSnapshot();
    });
});

describe('Testing formatInputs', () => {
    test('1 quantum register', () => {
        const inputs: Qubit[] = [{ id: 0 }];
        const expectedRegs: RegisterMap = { 0: { type: RegisterType.Qubit, y: startY } };
        const { qubitWires, registers } = formatInputs(inputs);
        expect(qubitWires).toMatchSnapshot();
        expect(registers).toEqual(expectedRegs);
        expect(registers).toEqual(expectedRegs);
    });
    test('Multiple quantum registers', () => {
        const inputs: Qubit[] = [{ id: 0 }, { id: 1 }, { id: 2 }];
        const expectedRegs: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            1: { type: RegisterType.Qubit, y: startY + registerHeight },
            2: { type: RegisterType.Qubit, y: startY + registerHeight * 2 },
        };
        const { qubitWires, registers } = formatInputs(inputs);
        expect(qubitWires).toMatchSnapshot();
        expect(registers).toEqual(expectedRegs);
        expect(registers).toEqual(expectedRegs);
    });
    test('Quantum and classical registers', () => {
        let inputs: Qubit[] = [{ id: 0, numChildren: 1 }, { id: 1 }, { id: 2 }];
        let expectedRegs: RegisterMap = {
            0: {
                type: RegisterType.Qubit,
                y: startY,
                children: [{ type: RegisterType.Classical, y: startY + classicalRegHeight }],
            },
            1: { type: RegisterType.Qubit, y: startY + classicalRegHeight * 2 },
            2: { type: RegisterType.Qubit, y: startY + registerHeight + classicalRegHeight * 2 },
        };
        let { qubitWires, registers } = formatInputs(inputs);
        expect(qubitWires).toMatchSnapshot();
        expect(registers).toEqual(expectedRegs);
        expect(registers).toEqual(expectedRegs);

        inputs = [{ id: 0 }, { id: 1, numChildren: 2 }, { id: 2, numChildren: 1 }];
        expectedRegs = {
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
        ({ qubitWires, registers } = formatInputs(inputs));
        expect(qubitWires).toMatchSnapshot();
        expect(registers).toEqual(expectedRegs);
        expect(registers).toEqual(expectedRegs);
    });
    test('Skip quantum registers', () => {
        const inputs: Qubit[] = [{ id: 0 }, { id: 2 }];
        const expectedRegs: RegisterMap = {
            0: { type: RegisterType.Qubit, y: startY },
            2: { type: RegisterType.Qubit, y: startY + registerHeight },
        };
        const { qubitWires, registers } = formatInputs(inputs);
        expect(qubitWires).toMatchSnapshot();
        expect(registers).toEqual(expectedRegs);
        expect(registers).toEqual(expectedRegs);
    });
});
