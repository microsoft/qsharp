// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { RegisterMap } from '../register';
import { regLineStart } from '../constants';
import { Metadata, GateType } from '../metadata';
import { group, line, text } from './formatUtils';

/**
 * Generate the SVG representation of the qubit register wires in `registers` and the classical wires
 * stemming from each measurement gate.
 *
 * @param registers    Map from register IDs to register metadata.
 * @param measureGates Array of measurement gates metadata.
 * @param endX         End x coord.
 *
 * @returns SVG representation of register wires.
 */
const formatRegisters = (registers: RegisterMap, measureGates: Metadata[], endX: number): SVGElement => {
    const formattedRegs: SVGElement[] = [];
    // Render qubit wires
    for (const qId in registers) {
        formattedRegs.push(_qubitRegister(Number(qId), endX, registers[qId].y));
    }
    // Render classical wires
    measureGates.forEach(({ type, x, targetsY, controlsY }) => {
        if (type !== GateType.Measure) return;
        const gateY: number = controlsY[0];
        (targetsY as number[]).forEach((y) => {
            formattedRegs.push(_classicalRegister(x, gateY, endX, y));
        });
    });
    return group(formattedRegs);
};

/**
 * Generates the SVG representation of a classical register.
 *
 * @param startX Start x coord.
 * @param gateY  y coord of measurement gate.
 * @param endX   End x coord.
 * @param wireY  y coord of wire.
 *
 * @returns SVG representation of the given classical register.
 */
const _classicalRegister = (startX: number, gateY: number, endX: number, wireY: number): SVGElement => {
    const wirePadding = 1;
    // Draw vertical lines
    const vLine1: SVGElement = line(
        startX + wirePadding,
        gateY,
        startX + wirePadding,
        wireY - wirePadding,
        'register-classical',
    );
    const vLine2: SVGElement = line(
        startX - wirePadding,
        gateY,
        startX - wirePadding,
        wireY + wirePadding,
        'register-classical',
    );

    // Draw horizontal lines
    const hLine1: SVGElement = line(
        startX + wirePadding,
        wireY - wirePadding,
        endX,
        wireY - wirePadding,
        'register-classical',
    );
    const hLine2: SVGElement = line(
        startX - wirePadding,
        wireY + wirePadding,
        endX,
        wireY + wirePadding,
        'register-classical',
    );

    return group([vLine1, vLine2, hLine1, hLine2]);
};

/**
 * Generates the SVG representation of a qubit register.
 *
 * @param qId         Qubit register index.
 * @param endX        End x coord.
 * @param y           y coord of wire.
 * @param labelOffset y offset for wire label.
 *
 * @returns SVG representation of the given qubit register.
 */
const _qubitRegister = (qId: number, endX: number, y: number, labelOffset = 16): SVGElement => {
    const wire: SVGElement = line(regLineStart, y, endX, y);

    const label: SVGElement = text(`q${qId}`, regLineStart, y - labelOffset);
    label.setAttribute('dominant-baseline', 'hanging');
    label.setAttribute('text-anchor', 'start');
    label.setAttribute('font-size', '75%');

    return group([wire, label]);
};

export { formatRegisters, _classicalRegister, _qubitRegister };
