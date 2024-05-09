import { getGateWidth, _getStringWidth, createUUID } from '../src/utils';
import { minGateWidth, labelPadding } from '../src/constants';
import { GateType } from '../src/metadata';

describe('Testing createUUID', () => {
    test('no x in uuid', () => expect(createUUID()).not.toContain('x'));
    test('no y in uuid', () => expect(createUUID()).not.toContain('y'));
});

describe('Testing getGateWidth', () => {
    test('measure gate', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Measure, label: '' }))).toEqual(minGateWidth));
    test('cnot gate', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Cnot, label: 'x' }))).toEqual(minGateWidth));
    test('swap gate', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Swap, label: '' }))).toEqual(minGateWidth));
    test('single unitary gate', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Unitary, label: 'x' }))).toEqual(minGateWidth));
    test('multi unitary gate', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Unitary, label: 'zz' }))).toEqual(minGateWidth));
    test('unitary gate with arguments', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Unitary, displayArgs: '(0.25)', label: 'RX' }))).toEqual(
            52,
        ));
    test('invalid', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Invalid, label: '' }))).toEqual(minGateWidth));
    test('unitary with long name', () =>
        expect(getGateWidth(Object.assign({ type: GateType.Unitary, label: 'FOOBAR' }))).toBeCloseTo(
            59 + labelPadding * 2,
        ));
    test('classically controlled gate', () => {
        expect(getGateWidth(Object.assign({ type: GateType.ClassicalControlled, label: '', width: 500 }))).toEqual(500);
    });
});

describe('Testing _getStringWidth', () => {
    test('correctly scaled width with font size', () => {
        expect(_getStringWidth('FOOBAR', 14)).toEqual(59);
        expect(_getStringWidth('FOOBAR', 20)).toEqual(84);
        expect(_getStringWidth('FOOBAR', 5)).toEqual(21);
        expect(_getStringWidth('FOOBAR', 100)).toEqual(423);
        expect(_getStringWidth('FOOBAR', 200)).toEqual(844);
    });

    test('varying size strings', () => {
        expect(_getStringWidth('H', 14)).toBeGreaterThanOrEqual(9);
        expect(_getStringWidth('H', 14)).toBeLessThanOrEqual(10);
        expect(_getStringWidth('GateWithASuperLongName', 14)).toBeGreaterThanOrEqual(174);
        expect(_getStringWidth('GateWithASuperLongName', 14)).toBeLessThanOrEqual(176);
    });
});
