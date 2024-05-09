// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { minGateWidth, startX, gatePadding, controlBtnOffset, groupBoxPadding } from './constants';
import { Operation, ConditionalRender } from './circuit';
import { Metadata, GateType } from './metadata';
import { Register, RegisterMap, RegisterType } from './register';
import { getGateWidth } from './utils';

/**
 * Takes in a list of operations and maps them to `metadata` objects which
 * contains information for formatting the corresponding SVG.
 *
 * @param operations Array of operations.
 * @param registers  Mapping from qubit IDs to register metadata.
 *
 * @returns An object containing `metadataList` (Array of Metadata objects) and
 *          `svgWidth` which is the width of the entire SVG.
 */
const processOperations = (
    operations: Operation[],
    registers: RegisterMap,
): { metadataList: Metadata[]; svgWidth: number } => {
    if (operations.length === 0) return { metadataList: [], svgWidth: startX };

    // Group operations based on registers
    const groupedOps: number[][] = _groupOperations(operations, registers);

    // Align operations on multiple registers
    const alignedOps: (number | null)[][] = _alignOps(groupedOps);

    // Maintain widths of each column to account for variable-sized gates
    const numColumns: number = Math.max(0, ...alignedOps.map((ops) => ops.length));
    const columnsWidths: number[] = new Array(numColumns).fill(minGateWidth);

    // Get classical registers and their starting column index
    const classicalRegs: [number, Register][] = _getClassicalRegStart(operations, alignedOps);

    // Keep track of which ops are already seen to avoid duplicate rendering
    const visited: { [opIdx: number]: boolean } = {};

    // Map operation index to gate metadata for formatting later
    const opsMetadata: Metadata[][] = alignedOps.map((regOps) =>
        regOps.map((opIdx, col) => {
            let op: Operation | null = null;

            if (opIdx != null && !visited.hasOwnProperty(opIdx)) {
                op = operations[opIdx];
                visited[opIdx] = true;
            }

            const metadata: Metadata = _opToMetadata(op, registers);

            if (op != null && [GateType.Unitary, GateType.ControlledUnitary].includes(metadata.type)) {
                // If gate is a unitary type, split targetsY into groups if there
                // is a classical register between them for rendering

                // Get y coordinates of classical registers in the same column as this operation
                const classicalRegY: number[] = classicalRegs
                    .filter(([regCol, _]) => regCol <= col)
                    .map(([_, reg]) => {
                        if (reg.cId == null) throw new Error('Could not find cId for classical register.');
                        const { children } = registers[reg.qId];
                        if (children == null)
                            throw new Error(`Failed to find classical registers for qubit ID ${reg.qId}.`);
                        return children[reg.cId].y;
                    });

                metadata.targetsY = _splitTargetsY(op.targets, classicalRegY, registers);
            }

            // Expand column size, if needed
            if (metadata.width > columnsWidths[col]) {
                columnsWidths[col] = metadata.width;
            }

            return metadata;
        }),
    );

    // Fill in x coord of each gate
    const endX: number = _fillMetadataX(opsMetadata, columnsWidths);

    // Flatten operations and filter out invalid gates
    const metadataList: Metadata[] = opsMetadata.flat().filter(({ type }) => type != GateType.Invalid);

    return { metadataList, svgWidth: endX };
};

/**
 * Group gates provided by operations into their respective registers.
 *
 * @param operations Array of operations.
 * @param numRegs    Total number of registers.
 *
 * @returns 2D array of indices where `groupedOps[i][j]` is the index of the operations
 *          at register `i` and column `j` (not yet aligned/padded).
 */
const _groupOperations = (operations: Operation[], registers: RegisterMap): number[][] => {
    // NOTE: We get the max ID instead of just number of keys because there can be a qubit ID that
    // isn't acted upon and thus does not show up as a key in registers.
    const numRegs: number = Math.max(-1, ...Object.keys(registers).map(Number)) + 1;
    const groupedOps: number[][] = Array.from(Array(numRegs), () => new Array(0));
    operations.forEach(({ targets, controls }, instrIdx) => {
        const ctrls: Register[] = controls || [];
        const qRegs: Register[] = [...ctrls, ...targets].filter(
            ({ type }) => (type || RegisterType.Qubit) === RegisterType.Qubit,
        );
        const qRegIdxList: number[] = qRegs.map(({ qId }) => qId);
        const clsControls: Register[] = ctrls.filter(
            ({ type }) => (type || RegisterType.Qubit) === RegisterType.Classical,
        );
        const isClassicallyControlled: boolean = clsControls.length > 0;
        if (!isClassicallyControlled && qRegs.length === 0) return;
        // If operation is classically-controlled, pad all qubit registers. Otherwise, only pad
        // the contiguous range of registers that it covers.
        const minRegIdx: number = isClassicallyControlled ? 0 : Math.min(...qRegIdxList);
        const maxRegIdx: number = isClassicallyControlled ? numRegs - 1 : Math.max(...qRegIdxList);
        // Add operation also to registers that are in-between target registers
        // so that other gates won't render in the middle.
        for (let i = minRegIdx; i <= maxRegIdx; i++) {
            groupedOps[i].push(instrIdx);
        }
    });
    return groupedOps;
};

/**
 * Aligns operations by padding registers with `null`s to make sure that multiqubit
 * gates are in the same column.
 * e.g. ---[x]---[x]--
 *      ----------|---
 *
 * @param ops 2D array of operations. Each row represents a register
 *            and the operations acting on it (in-order).
 *
 * @returns 2D array of aligned operations padded with `null`s.
 */
const _alignOps = (ops: number[][]): (number | null)[][] => {
    let maxNumOps: number = Math.max(0, ...ops.map((regOps) => regOps.length));
    let col = 0;
    // Deep copy ops to be returned as paddedOps
    const paddedOps: (number | null)[][] = JSON.parse(JSON.stringify(ops));
    while (col < maxNumOps) {
        for (let regIdx = 0; regIdx < paddedOps.length; regIdx++) {
            const reg: (number | null)[] = paddedOps[regIdx];
            if (reg.length <= col) continue;

            // Should never be null (nulls are only padded to previous columns)
            const opIdx: number | null = reg[col];

            // Get position of gate
            const targetsPos: number[] = paddedOps.map((regOps) => regOps.indexOf(opIdx));
            const gatePos: number = Math.max(-1, ...targetsPos);

            // If current column is not desired gate position, pad with null
            if (col < gatePos) {
                paddedOps[regIdx].splice(col, 0, null);
                maxNumOps = Math.max(maxNumOps, paddedOps[regIdx].length);
            }
        }
        col++;
    }
    return paddedOps;
};

/**
 * Retrieves the starting index of each classical register.
 *
 * @param ops     Array of operations.
 * @param idxList 2D array of aligned operation indices.
 *
 * @returns Array of classical register and their starting column indices in the form [[column, register]].
 */
const _getClassicalRegStart = (ops: Operation[], idxList: (number | null)[][]): [number, Register][] => {
    const clsRegs: [number, Register][] = [];
    idxList.forEach((reg) => {
        for (let col = 0; col < reg.length; col++) {
            const opIdx: number | null = reg[col];
            if (opIdx != null && ops[opIdx].isMeasurement) {
                const targetClsRegs: Register[] = ops[opIdx].targets.filter(
                    (reg) => reg.type === RegisterType.Classical,
                );
                targetClsRegs.forEach((reg) => clsRegs.push([col, reg]));
            }
        }
    });
    return clsRegs;
};

/**
 * Maps operation to metadata (e.g. gate type, position, dimensions, text)
 * required to render the image.
 *
 * @param op        Operation to be mapped into metadata format.
 * @param registers Array of registers.
 *
 * @returns Metadata representation of given operation.
 */
const _opToMetadata = (op: Operation | null, registers: RegisterMap): Metadata => {
    const metadata: Metadata = {
        type: GateType.Invalid,
        x: 0,
        controlsY: [],
        targetsY: [],
        label: '',
        width: -1,
    };

    if (op == null) return metadata;

    const {
        gate,
        dataAttributes,
        displayArgs,
        isMeasurement,
        isConditional,
        isControlled,
        isAdjoint,
        controls,
        targets,
        children,
        conditionalRender,
    } = op;

    // Set y coords
    metadata.controlsY = controls?.map((reg) => _getRegY(reg, registers)) || [];
    metadata.targetsY = targets.map((reg) => _getRegY(reg, registers));

    if (isConditional) {
        // Classically-controlled operations
        if (children == null || children.length == 0)
            throw new Error('No children operations found for classically-controlled operation.');

        // Gates to display when classical bit is 0.
        const onZeroOps: Operation[] = children.filter((op) => op.conditionalRender !== ConditionalRender.OnOne);
        let childrenInstrs = processOperations(onZeroOps, registers);
        const zeroGates: Metadata[] = childrenInstrs.metadataList;
        const zeroChildWidth: number = childrenInstrs.svgWidth;

        // Gates to display when classical bit is 1.
        const onOneOps: Operation[] = children.filter((op) => op.conditionalRender !== ConditionalRender.OnZero);
        childrenInstrs = processOperations(onOneOps, registers);
        const oneGates: Metadata[] = childrenInstrs.metadataList;
        const oneChildWidth: number = childrenInstrs.svgWidth;

        // Subtract startX (left-side) and 2*gatePadding (right-side) from nested child gates width
        const width: number = Math.max(zeroChildWidth, oneChildWidth) - startX - gatePadding * 2;

        metadata.type = GateType.ClassicalControlled;
        metadata.children = [zeroGates, oneGates];
        // Add additional width from control button and inner box padding for dashed box
        metadata.width = width + controlBtnOffset + groupBoxPadding * 2;

        // Set targets to first and last quantum registers so we can render the surrounding box
        // around all quantum registers.
        const qubitsY: number[] = Object.values(registers).map(({ y }) => y);
        if (qubitsY.length > 0) metadata.targetsY = [Math.min(...qubitsY), Math.max(...qubitsY)];
    } else if (conditionalRender == ConditionalRender.AsGroup && (children?.length || 0) > 0) {
        const childrenInstrs = processOperations(children as Operation[], registers);
        metadata.type = GateType.Group;
        metadata.children = childrenInstrs.metadataList;
        // _zoomButton function in gateFormatter.ts relies on
        // 'expanded' attribute to render zoom button
        metadata.dataAttributes = { expanded: 'true' };
        // Subtract startX (left-side) and add inner box padding (minus nested gate padding) for dashed box
        metadata.width = childrenInstrs.svgWidth - startX + (groupBoxPadding - gatePadding) * 2;
    } else if (isMeasurement) {
        metadata.type = GateType.Measure;
    } else if (gate === 'SWAP') {
        metadata.type = GateType.Swap;
    } else if (isControlled) {
        metadata.type = gate === 'X' ? GateType.Cnot : GateType.ControlledUnitary;
        metadata.label = gate;
    } else if (gate === 'X') {
        metadata.type = GateType.X;
        metadata.label = gate;
    } else {
        // Any other gate treated as a simple unitary gate
        metadata.type = GateType.Unitary;
        metadata.label = gate;
    }

    // If adjoint, add ' to the end of gate label
    if (isAdjoint && metadata.label.length > 0) metadata.label += "'";

    // If gate has extra arguments, display them
    if (displayArgs != null) metadata.displayArgs = displayArgs;

    // Set gate width
    metadata.width = getGateWidth(metadata);

    // Extend existing data attributes with user-provided data attributes
    if (dataAttributes != null) metadata.dataAttributes = { ...metadata.dataAttributes, ...dataAttributes };

    return metadata;
};

/**
 * Compute the y coord of a given register.
 *
 * @param reg       Register to compute y coord of.
 * @param registers Map of qubit IDs to RegisterMetadata.
 *
 * @returns The y coord of give register.
 */
const _getRegY = (reg: Register, registers: RegisterMap): number => {
    const { type, qId, cId } = reg;
    if (!registers.hasOwnProperty(qId)) throw new Error(`ERROR: Qubit register with ID ${qId} not found.`);
    const { y, children } = registers[qId];
    switch (type) {
        case undefined:
        case RegisterType.Qubit:
            return y;
        case RegisterType.Classical:
            if (children == null) throw new Error(`ERROR: No classical registers found for qubit ID ${qId}.`);
            if (cId == null)
                throw new Error(`ERROR: No ID defined for classical register associated with qubit ID ${qId}.`);
            if (children.length <= cId)
                throw new Error(
                    `ERROR: Classical register ID ${cId} invalid for qubit ID ${qId} with ${children.length} classical register(s).`,
                );
            return children[cId].y;
        default:
            throw new Error(`ERROR: Unknown register type ${type}.`);
    }
};

/**
 * Splits `targets` if non-adjacent or intersected by classical registers.
 *
 * @param targets       Target qubit registers.
 * @param classicalRegY y coords of classical registers overlapping current column.
 * @param registers     Mapping from register qubit IDs to register metadata.
 *
 * @returns Groups of target qubit y coords.
 */
const _splitTargetsY = (targets: Register[], classicalRegY: number[], registers: RegisterMap): number[][] => {
    if (targets.length === 0) return [];

    // Get qIds sorted by ascending y value
    const orderedQIds: number[] = Object.keys(registers).map(Number);
    orderedQIds.sort((a, b) => registers[a].y - registers[b].y);
    const qIdPosition: { [qId: number]: number } = {};
    orderedQIds.forEach((qId, i) => (qIdPosition[qId] = i));

    // Sort targets and classicalRegY by ascending y value
    targets = targets.slice();
    targets.sort((a, b) => {
        const posDiff: number = qIdPosition[a.qId] - qIdPosition[b.qId];
        if (posDiff === 0 && a.cId != null && b.cId != null) return a.cId - b.cId;
        else return posDiff;
    });
    classicalRegY = classicalRegY.slice();
    classicalRegY.sort((a, b) => a - b);

    let prevPos = 0;
    let prevY = 0;

    return targets.reduce((groups: number[][], target: Register) => {
        const y = _getRegY(target, registers);
        const pos = qIdPosition[target.qId];

        // Split into new group if one of the following holds:
        //      1. First target register
        //      2. Non-adjacent qubit registers
        //      3. There is a classical register between current and previous register
        if (groups.length === 0 || pos > prevPos + 1 || (classicalRegY[0] > prevY && classicalRegY[0] < y))
            groups.push([y]);
        else groups[groups.length - 1].push(y);

        prevPos = pos;
        prevY = y;

        // Remove classical registers that are higher than current y
        while (classicalRegY.length > 0 && classicalRegY[0] <= y) classicalRegY.shift();

        return groups;
    }, []);
};

/**
 * Updates the x coord of each metadata in the given 2D array of metadata and returns rightmost x coord.
 *
 * @param opsMetadata  2D array of metadata.
 * @param columnWidths Array of column widths.
 *
 * @returns Rightmost x coord.
 */
const _fillMetadataX = (opsMetadata: Metadata[][], columnWidths: number[]): number => {
    let currX: number = startX;

    const colStartX: number[] = columnWidths.map((width) => {
        const x: number = currX;
        currX += width + gatePadding * 2;
        return x;
    });

    const endX: number = currX;

    opsMetadata.forEach((regOps) =>
        regOps.forEach((metadata, col) => {
            const x = colStartX[col];
            switch (metadata.type) {
                case GateType.ClassicalControlled:
                case GateType.Group:
                    // Subtract startX offset from nested gates and add offset and padding
                    let offset: number = x - startX + groupBoxPadding;
                    if (metadata.type === GateType.ClassicalControlled) offset += controlBtnOffset;

                    // Offset each x coord in children gates
                    _offsetChildrenX(metadata.children, offset);

                    // We don't use the centre x coord because we only care about the rightmost x for
                    // rendering the box around the group of nested gates
                    metadata.x = x;
                    break;

                default:
                    metadata.x = x + columnWidths[col] / 2;
                    break;
            }
        }),
    );

    return endX;
};

/**
 * Offset x coords of nested children operations.
 *
 * @param children 2D array of children metadata.
 * @param offset   x coord offset.
 */
const _offsetChildrenX = (children: (Metadata | Metadata[])[] | undefined, offset: number): void => {
    if (children == null) return;
    children.flat().forEach((child) => {
        child.x += offset;
        _offsetChildrenX(child.children, offset);
    });
};

export {
    processOperations,
    _groupOperations,
    _alignOps,
    _getClassicalRegStart,
    _opToMetadata,
    _getRegY,
    _splitTargetsY,
    _fillMetadataX,
    _offsetChildrenX,
};
