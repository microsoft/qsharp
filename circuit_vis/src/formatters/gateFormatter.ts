// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Metadata, GateType } from '../metadata';
import {
    minGateWidth,
    gateHeight,
    labelFontSize,
    argsFontSize,
    controlBtnRadius,
    controlBtnOffset,
    groupBoxPadding,
    classicalRegHeight,
    nestedGroupPadding,
} from '../constants';
import {
    createSvgElement,
    group,
    line,
    circle,
    controlDot,
    box,
    text,
    arc,
    dashedLine,
    dashedBox,
} from './formatUtils';

/**
 * Given an array of operations (in metadata format), return the SVG representation.
 *
 * @param opsMetadata Array of Metadata representation of operations.
 * @param nestedDepth Depth of nested operations (used in classically controlled and grouped operations).
 *
 * @returns SVG representation of operations.
 */
const formatGates = (opsMetadata: Metadata[], nestedDepth = 0): SVGElement => {
    const formattedGates: SVGElement[] = opsMetadata.map((metadata) => _formatGate(metadata, nestedDepth));
    return group(formattedGates);
};

/**
 * Takes in an operation's metadata and formats it into SVG.
 *
 * @param metadata Metadata object representation of gate.
 * @param nestedDepth Depth of nested operations (used in classically controlled and grouped operations).
 *
 * @returns SVG representation of gate.
 */
const _formatGate = (metadata: Metadata, nestedDepth = 0): SVGElement => {
    const { type, x, controlsY, targetsY, label, displayArgs, width } = metadata;
    switch (type) {
        case GateType.Measure:
            return _createGate([_measure(x, controlsY[0])], metadata, nestedDepth);
        case GateType.Unitary:
            return _createGate([_unitary(label, x, targetsY as number[][], width, displayArgs)], metadata, nestedDepth);
        case GateType.X:
            return _createGate([_x(metadata, nestedDepth)], metadata, nestedDepth);
        case GateType.Swap:
            return controlsY.length > 0
                ? _controlledGate(metadata, nestedDepth)
                : _createGate([_swap(metadata, nestedDepth)], metadata, nestedDepth);
        case GateType.Cnot:
        case GateType.ControlledUnitary:
            return _controlledGate(metadata, nestedDepth);
        case GateType.Group:
            return _groupedOperations(metadata, nestedDepth);
        case GateType.ClassicalControlled:
            return _classicalControlled(metadata);
        default:
            throw new Error(`ERROR: unknown gate (${label}) of type ${type}.`);
    }
};

/**
 * Groups SVG elements into a gate SVG group.
 *
 * @param svgElems       Array of SVG elements.
 * @param dataAttributes Custom data attributes to be attached to SVG group.
 *
 * @returns SVG representation of a gate.
 */
const _createGate = (svgElems: SVGElement[], metadata: Metadata, nestedDepth: number): SVGElement => {
    const { dataAttributes } = metadata || {};
    const attributes: { [attr: string]: string } = { class: 'gate' };
    Object.entries(dataAttributes || {}).forEach(([attr, val]) => (attributes[`data-${attr}`] = val));

    const zoomBtn: SVGElement | null = _zoomButton(metadata, nestedDepth);
    if (zoomBtn != null) svgElems = svgElems.concat([zoomBtn]);
    return group(svgElems, attributes);
};

/**
 * Returns the expand/collapse button for an operation if it can be zoomed-in or zoomed-out,
 * respectively. If neither are allowed, return `null`.
 *
 * @param metadata Operation metadata.
 * @param nestedDepth Depth of nested operation.
 *
 * @returns SVG element for expand/collapse button if needed, or null otherwise.
 */
const _zoomButton = (metadata: Metadata, nestedDepth: number): SVGElement | null => {
    if (metadata == undefined) return null;

    const [x1, y1] = _gatePosition(metadata, nestedDepth);
    let { dataAttributes } = metadata;
    dataAttributes = dataAttributes || {};

    const expanded = 'expanded' in dataAttributes;

    const x = x1 + 2;
    const y = y1 + 2;
    const circleBorder: SVGElement = circle(x, y, 10);

    if (expanded) {
        // Create collapse button if expanded
        const minusSign: SVGElement = createSvgElement('path', { d: `M${x - 7},${y} h14` });
        const elements: SVGElement[] = [circleBorder, minusSign];
        return group(elements, { class: 'gate-control gate-collapse' });
    } else if (dataAttributes['zoom-in'] == 'true') {
        // Create expand button if operation can be zoomed in
        const plusSign: SVGElement = createSvgElement('path', { d: `M${x},${y - 7} v14 M${x - 7},${y} h14` });
        const elements: SVGElement[] = [circleBorder, plusSign];
        return group(elements, { class: 'gate-control gate-expand' });
    }

    return null;
};

/**
 * Calculate position of gate.
 *
 * @param metadata Operation metadata.
 * @param nestedDepth Depth of nested operations.
 *
 * @returns Coordinates of gate: [x1, y1, x2, y2].
 */
const _gatePosition = (metadata: Metadata, nestedDepth: number): [number, number, number, number] => {
    const { x, width, type, targetsY } = metadata;

    const ys = targetsY?.flatMap((y) => y as number[]) || [];
    const maxY = Math.max(...ys);
    const minY = Math.min(...ys);

    let x1: number, y1: number, x2: number, y2: number;

    switch (type) {
        case GateType.Group:
            const padding = groupBoxPadding - nestedDepth * nestedGroupPadding;

            x1 = x - 2 * padding;
            y1 = minY - gateHeight / 2 - padding;
            x2 = width + 2 * padding;
            y2 = maxY + +gateHeight / 2 + padding - (minY - gateHeight / 2 - padding);

            return [x1, y1, x2, y2];

        default:
            x1 = x - width / 2;
            y1 = minY - gateHeight / 2;
            x2 = x + width;
            y2 = maxY + gateHeight / 2;
    }

    return [x1, y1, x2, y2];
};

/**
 * Creates a measurement gate at position (x, y).
 *
 * @param x  x coord of measurement gate.
 * @param y  y coord of measurement gate.
 *
 * @returns SVG representation of measurement gate.
 */
const _measure = (x: number, y: number): SVGElement => {
    x -= minGateWidth / 2;
    const width: number = minGateWidth,
        height = gateHeight;
    // Draw measurement box
    const mBox: SVGElement = box(x, y - height / 2, width, height, 'gate-measure');
    const mArc: SVGElement = arc(x + 5, y + 2, width / 2 - 5, height / 2 - 8);
    const meter: SVGElement = line(x + width / 2, y + 8, x + width - 8, y - height / 2 + 8);
    return group([mBox, mArc, meter]);
};

/**
 * Creates the SVG for a unitary gate on an arbitrary number of qubits.
 *
 * @param label            Gate label.
 * @param x                x coord of gate.
 * @param y                Array of y coords of registers acted upon by gate.
 * @param width            Width of gate.
 * @param displayArgs           Arguments passed in to gate.
 * @param renderDashedLine If true, draw dashed lines between non-adjacent unitaries.
 *
 * @returns SVG representation of unitary gate.
 */
const _unitary = (
    label: string,
    x: number,
    y: number[][],
    width: number,
    displayArgs?: string,
    renderDashedLine = true,
): SVGElement => {
    if (y.length === 0) throw new Error(`Failed to render unitary gate (${label}): has no y-values`);

    // Render each group as a separate unitary boxes
    const unitaryBoxes: SVGElement[] = y.map((group: number[]) => {
        const maxY: number = group[group.length - 1],
            minY: number = group[0];
        const height: number = maxY - minY + gateHeight;
        return _unitaryBox(label, x, minY, width, height, displayArgs);
    });

    // Draw dashed line between disconnected unitaries
    if (renderDashedLine && unitaryBoxes.length > 1) {
        const lastBox: number[] = y[y.length - 1];
        const firstBox: number[] = y[0];
        const maxY: number = lastBox[lastBox.length - 1],
            minY: number = firstBox[0];
        const vertLine: SVGElement = dashedLine(x, minY, x, maxY);
        return group([vertLine, ...unitaryBoxes]);
    }

    return group(unitaryBoxes);
};

/**
 * Generates SVG representation of the boxed unitary gate symbol.
 *
 * @param label  Label for unitary operation.
 * @param x      x coord of gate.
 * @param y      y coord of gate.
 * @param width  Width of gate.
 * @param height Height of gate.
 * @param displayArgs Arguments passed in to gate.
 *
 * @returns SVG representation of unitary box.
 */
const _unitaryBox = (
    label: string,
    x: number,
    y: number,
    width: number,
    height: number = gateHeight,
    displayArgs?: string,
): SVGElement => {
    y -= gateHeight / 2;
    const uBox: SVGElement = box(x - width / 2, y, width, height);
    const labelY = y + height / 2 - (displayArgs == null ? 0 : 7);
    const labelText: SVGElement = text(label, x, labelY);
    const elems = [uBox, labelText];
    if (displayArgs != null) {
        const argStrY = y + height / 2 + 8;
        const argText: SVGElement = text(displayArgs, x, argStrY, argsFontSize);
        elems.push(argText);
    }
    return group(elems);
};

/**
 * Creates the SVG for a SWAP gate on y coords given by targetsY.
 *
 * @param x          Centre x coord of SWAP gate.
 * @param targetsY   y coords of target registers.
 *
 * @returns SVG representation of SWAP gate.
 */
const _swap = (metadata: Metadata, nestedDepth: number): SVGElement => {
    const { x, targetsY } = metadata;

    // Get SVGs of crosses
    const [x1, y1, x2, y2] = _gatePosition(metadata, nestedDepth);
    const ys = targetsY?.flatMap((y) => y as number[]) || [];

    const bg: SVGElement = box(x1, y1, x2, y2, 'gate-swap');
    const crosses: SVGElement[] = ys.map((y) => _cross(x, y));
    const vertLine: SVGElement = line(x, ys[0], x, ys[1]);
    return group([bg, ...crosses, vertLine]);
};
/**
 * Creates the SVG for an X gate
 *
 * @returns SVG representation of X gate.
 */
const _x = (metadata: Metadata, _: number): SVGElement => {
    const { x, targetsY } = metadata;
    const ys = targetsY.flatMap((y) => y as number[]);
    return _oplus(x, ys[0]);
};
/**
 * Generates cross for display in SWAP gate.
 *
 * @param x x coord of gate.
 * @param y y coord of gate.
 *
 * @returns SVG representation for cross.
 */
const _cross = (x: number, y: number): SVGElement => {
    const radius = 8;
    const line1: SVGElement = line(x - radius, y - radius, x + radius, y + radius);
    const line2: SVGElement = line(x - radius, y + radius, x + radius, y - radius);
    return group([line1, line2]);
};

/**
 * Produces the SVG representation of a controlled gate on multiple qubits.
 *
 * @param metadata Metadata of controlled gate.
 *
 * @returns SVG representation of controlled gate.
 */
const _controlledGate = (metadata: Metadata, nestedDepth: number): SVGElement => {
    const targetGateSvgs: SVGElement[] = [];
    const { type, x, controlsY, label, displayArgs, width } = metadata;
    let { targetsY } = metadata;

    // Get SVG for target gates
    switch (type) {
        case GateType.Cnot:
            (targetsY as number[]).forEach((y) => targetGateSvgs.push(_oplus(x, y)));
            break;
        case GateType.Swap:
            (targetsY as number[]).forEach((y) => targetGateSvgs.push(_cross(x, y)));
            break;
        case GateType.ControlledUnitary:
            const groupedTargetsY: number[][] = targetsY as number[][];
            targetGateSvgs.push(_unitary(label, x, groupedTargetsY, width, displayArgs, false));
            targetsY = targetsY.flat();
            break;
        default:
            throw new Error(`ERROR: Unrecognized gate: ${label} of type ${type}`);
    }
    // Get SVGs for control dots
    const controlledDotsSvg: SVGElement[] = controlsY.map((y) => controlDot(x, y));
    // Create control lines
    const maxY: number = Math.max(...controlsY, ...(targetsY as number[]));
    const minY: number = Math.min(...controlsY, ...(targetsY as number[]));
    const vertLine: SVGElement = line(x, minY, x, maxY);
    const svg: SVGElement = _createGate([vertLine, ...controlledDotsSvg, ...targetGateSvgs], metadata, nestedDepth);
    return svg;
};

/**
 * Generates $\oplus$ symbol for display in CNOT gate.
 *
 * @param x x coordinate of gate.
 * @param y y coordinate of gate.
 * @param r radius of circle.
 *
 * @returns SVG representation of $\oplus$ symbol.
 */
const _oplus = (x: number, y: number, r = 15): SVGElement => {
    const circleBorder: SVGElement = circle(x, y, r);
    const vertLine: SVGElement = line(x, y - r, x, y + r);
    const horLine: SVGElement = line(x - r, y, x + r, y);
    return group([circleBorder, vertLine, horLine], { class: 'oplus' });
};

/**
 * Generates the SVG for a group of nested operations.
 *
 * @param metadata Metadata representation of gate.
 * @param nestedDepth Depth of nested operations (used in classically controlled and grouped operations).
 *
 * @returns SVG representation of gate.
 */
const _groupedOperations = (metadata: Metadata, nestedDepth: number): SVGElement => {
    const { children } = metadata;
    const [x1, y1, x2, y2] = _gatePosition(metadata, nestedDepth);

    // Draw dashed box around children gates
    const box: SVGElement = dashedBox(x1, y1, x2, y2);
    const elems: SVGElement[] = [box];
    if (children != null) elems.push(formatGates(children as Metadata[], nestedDepth + 1));
    return _createGate(elems, metadata, nestedDepth);
};

/**
 * Generates the SVG for a classically controlled group of operations.
 *
 * @param metadata Metadata representation of gate.
 * @param padding  Padding within dashed box.
 *
 * @returns SVG representation of gate.
 */
const _classicalControlled = (metadata: Metadata, padding: number = groupBoxPadding): SVGElement => {
    const { controlsY, dataAttributes } = metadata;
    const targetsY: number[] = metadata.targetsY as number[];
    const children: Metadata[][] = metadata.children as Metadata[][];
    let { x, width } = metadata;

    const controlY = controlsY[0];

    const elems: SVGElement[] = [];

    if (children != null) {
        if (children.length !== 2)
            throw new Error(`Invalid number of children found for classically-controlled gate: ${children.length}`);

        // Get SVG for gates controlled on 0
        const childrenZero: SVGElement = formatGates(children[0]);
        childrenZero.setAttribute('class', 'gates-zero');
        elems.push(childrenZero);

        // Get SVG for gates controlled on 1
        const childrenOne: SVGElement = formatGates(children[1]);
        childrenOne.setAttribute('class', 'gates-one');
        elems.push(childrenOne);
    }

    // Draw control button and attached dashed line to dashed box
    const controlCircleX: number = x + controlBtnRadius;
    const controlCircle: SVGElement = _controlCircle(controlCircleX, controlY);
    const lineY1: number = controlY + controlBtnRadius,
        lineY2: number = controlY + classicalRegHeight / 2;
    const vertLine: SVGElement = dashedLine(controlCircleX, lineY1, controlCircleX, lineY2, 'classical-line');
    x += controlBtnOffset;
    const horLine: SVGElement = dashedLine(controlCircleX, lineY2, x, lineY2, 'classical-line');

    width = width - controlBtnOffset + (padding - groupBoxPadding) * 2;
    x += groupBoxPadding - padding;
    const y: number = targetsY[0] - gateHeight / 2 - padding;
    const height: number = targetsY[1] - targetsY[0] + gateHeight + padding * 2;

    // Draw dashed box around children gates
    const box: SVGElement = dashedBox(x, y, width, height, 'classical-container');

    elems.push(...[horLine, vertLine, controlCircle, box]);

    // Display controlled operation in initial "unknown" state
    const attributes: { [attr: string]: string } = {
        class: `classically-controlled-group classically-controlled-unknown`,
    };
    if (dataAttributes != null)
        Object.entries(dataAttributes).forEach(([attr, val]) => (attributes[`data-${attr}`] = val));

    return group(elems, attributes);
};

/**
 * Generates the SVG representation of the control circle on a classical register with interactivity support
 * for toggling between bit values (unknown, 1, and 0).
 *
 * @param x   x coord.
 * @param y   y coord.
 * @param r   Radius of circle.
 *
 * @returns SVG representation of control circle.
 */
const _controlCircle = (x: number, y: number, r: number = controlBtnRadius): SVGElement =>
    group([circle(x, y, r), text('?', x, y, labelFontSize)], { class: 'classically-controlled-btn' });

export {
    formatGates,
    _formatGate,
    _createGate,
    _zoomButton,
    _measure,
    _unitary,
    _swap,
    _controlledGate,
    _groupedOperations,
    _classicalControlled,
};
