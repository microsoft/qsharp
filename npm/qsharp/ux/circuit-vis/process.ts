// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import {
  minGateWidth,
  startX,
  gatePadding,
  controlBtnOffset,
  groupBoxPadding,
} from "./constants";
import { ComponentGrid, Operation, ConditionalRender } from "./circuit";
import { Metadata, GateType } from "./metadata";
import { Register, RegisterMap } from "./register";
import { getGateWidth } from "./utils";

/**
 * Takes in a component grid and maps the operations to `metadata` objects which
 * contains information for formatting the corresponding SVG.
 *
 * @param componentGrid Grid of circuit components.
 * @param registers  Mapping from qubit IDs to register metadata.
 *
 * @returns An object containing `metadataArray` (2D Array of Metadata objects) and
 *          `svgWidth` which is the width of the entire SVG.
 */
const processOperations = (
  componentGrid: ComponentGrid,
  registers: RegisterMap,
): { metadataArray: Metadata[][]; svgWidth: number } => {
  if (componentGrid.length === 0)
    return { metadataArray: [], svgWidth: startX };
  const numColumns: number = componentGrid.length;
  const columnsWidths: number[] = new Array(numColumns).fill(minGateWidth);

  // Get classical registers and their starting column index
  const classicalRegs: [number, Register][] =
    _getClassicalRegStart(componentGrid);

  // Map operation index to gate metadata for formatting later
  const opsMetadata: Metadata[][] = componentGrid.map((col, colIndex) =>
    col.components.map((op) => {
      const metadata: Metadata = _opToMetadata(op, registers);

      if (
        op != null &&
        [GateType.Unitary, GateType.Ket, GateType.ControlledUnitary].includes(
          metadata.type,
        )
      ) {
        // If gate is a unitary type, split targetsY into groups if there
        // is a classical register between them for rendering

        // Get y coordinates of classical registers in the same column as this operation
        const classicalRegY: number[] = classicalRegs
          .filter(([regCol]) => regCol <= colIndex)
          .map(([, reg]) => {
            if (reg.result == null)
              throw new Error("Could not find cId for classical register.");
            const { children } = registers[reg.qubit];
            if (children == null)
              throw new Error(
                `Failed to find classical registers for qubit ID ${reg.qubit}.`,
              );
            return children[reg.result].y;
          });

        let qubits: Register[];
        switch (op.kind) {
          case "unitary":
            qubits = op.targets;
            break;
          case "measurement":
            qubits = op.qubits;
            break;
          case "ket":
            qubits = op.targets;
            break;
        }

        metadata.targetsY = _splitTargetsY(qubits, classicalRegY, registers);
      }

      // Expand column size, if needed
      if (metadata.width > columnsWidths[colIndex]) {
        columnsWidths[colIndex] = metadata.width;
      }

      return metadata;
    }),
  );

  // Filter out invalid gates
  const metadataArray: Metadata[][] = opsMetadata
    .map((col) => col.filter(({ type }) => type != GateType.Invalid))
    .filter((col) => col.length > 0);

  // Fill in x coord of each gate
  const endX: number = _fillMetadataX(metadataArray, columnsWidths);

  return { metadataArray, svgWidth: endX };
};

/**
 * Retrieves the starting index of each classical register.
 *
 * @param componentGrid Grid of circuit components.
 *
 * @returns Array of classical register and their starting column indices in the form [[column, register]].
 */
const _getClassicalRegStart = (
  componentGrid: ComponentGrid,
): [number, Register][] => {
  const clsRegs: [number, Register][] = [];
  componentGrid.forEach((col, colIndex) => {
    col.components.forEach((op) => {
      if (op.kind === "measurement") {
        const resultRegs: Register[] = op.results.filter(
          ({ result }) => result !== undefined,
        );
        resultRegs.forEach((reg) => clsRegs.push([colIndex, reg]));
      }
    });
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
const _opToMetadata = (
  op: Operation | null,
  registers: RegisterMap,
): Metadata => {
  const metadata: Metadata = {
    type: GateType.Invalid,
    x: 0,
    controlsY: [],
    targetsY: [],
    label: "",
    width: -1,
  };

  if (op == null) return metadata;

  let isAdjoint: boolean;
  let controls: Register[] | undefined;
  let targets: Register[];
  switch (op.kind) {
    case "measurement":
      isAdjoint = false;
      controls = op.qubits;
      targets = op.results;
      break;
    case "unitary":
      isAdjoint = op.isAdjoint ?? false;
      controls = op.controls;
      targets = op.targets;
      break;
    case "ket":
      isAdjoint = false;
      controls = [];
      targets = op.targets;
      break;
  }

  const {
    gate,
    args,
    children,
    dataAttributes,
    isConditional,
    conditionalRender,
  } = op;

  // Set y coords
  metadata.controlsY = controls?.map((reg) => _getRegY(reg, registers)) || [];
  metadata.targetsY = targets.map((reg) => _getRegY(reg, registers));

  if (isConditional) {
    // Classically-controlled operations
    if (children == null || children.length == 0)
      throw new Error(
        "No children operations found for classically-controlled operation.",
      );

    // Gates to display when classical bit is 0.
    const onZeroOps: ComponentGrid = children
      .map((col) => ({
        components: col.components.filter(
          (op) => op.conditionalRender === ConditionalRender.OnZero,
        ),
      }))
      .filter((col) => col.components.length > 0);
    let childrenInstrs = processOperations(onZeroOps, registers);
    const zeroGates: Metadata[][] = childrenInstrs.metadataArray;
    const zeroChildWidth: number = childrenInstrs.svgWidth;

    // Gates to display when classical bit is 1.
    const onOneOps: ComponentGrid = children
      .map((col) => ({
        components: col.components.filter(
          (op) => op.conditionalRender !== ConditionalRender.OnZero,
        ),
      }))
      .filter((col) => col.components.length > 0);
    childrenInstrs = processOperations(onOneOps, registers);
    const oneGates: Metadata[][] = childrenInstrs.metadataArray;
    const oneChildWidth: number = childrenInstrs.svgWidth;

    // Subtract startX (left-side) and 2*gatePadding (right-side) from nested child gates width
    const width: number =
      Math.max(zeroChildWidth, oneChildWidth) - startX - gatePadding * 2;

    metadata.type = GateType.ClassicalControlled;
    metadata.children = [zeroGates, oneGates];
    // Add additional width from control button and inner box padding for dashed box
    metadata.width = width + controlBtnOffset + groupBoxPadding * 2;

    // Set targets to first and last quantum registers so we can render the surrounding box
    // around all quantum registers.
    const qubitsY: number[] = Object.values(registers).map(({ y }) => y);
    if (qubitsY.length > 0)
      metadata.targetsY = [Math.min(...qubitsY), Math.max(...qubitsY)];
  } else if (
    conditionalRender == ConditionalRender.AsGroup &&
    (children?.length || 0) > 0
  ) {
    const childrenInstrs = processOperations(children!, registers);
    metadata.type = GateType.Group;
    metadata.children = childrenInstrs.metadataArray;
    // _zoomButton function in gateFormatter.ts relies on
    // 'expanded' attribute to render zoom button
    metadata.dataAttributes = { expanded: "true" };
    // Subtract startX (left-side) and add inner box padding (minus nested gate padding) for dashed box
    metadata.width =
      childrenInstrs.svgWidth - startX + (groupBoxPadding - gatePadding) * 2;
  } else if (op.kind === "measurement") {
    metadata.type = GateType.Measure;
  } else if (op.kind === "ket") {
    metadata.type = GateType.Ket;
    metadata.label = gate;
  } else if (gate === "SWAP") {
    metadata.type = GateType.Swap;
  } else if (controls && controls.length > 0) {
    metadata.type = gate === "X" ? GateType.Cnot : GateType.ControlledUnitary;
    metadata.label = gate;
  } else if (gate === "X") {
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
  // For now, we only display the first argument
  if (args !== undefined && args.length > 0) metadata.displayArgs = args[0];

  // Set gate width
  metadata.width = getGateWidth(metadata);

  // Extend existing data attributes with user-provided data attributes
  if (dataAttributes != null)
    metadata.dataAttributes = { ...metadata.dataAttributes, ...dataAttributes };

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
  const { qubit: qId, result } = reg;
  if (!Object.prototype.hasOwnProperty.call(registers, qId))
    throw new Error(`ERROR: Qubit register with ID ${qId} not found.`);
  const { y, children } = registers[qId];
  if (result == null) {
    return y;
  } else {
    if (children == null)
      throw new Error(
        `ERROR: No classical registers found for qubit ID ${qId}.`,
      );
    if (children.length <= result)
      throw new Error(
        `ERROR: Classical register ID ${result} invalid for qubit ID ${qId} with ${children.length} classical register(s).`,
      );
    return children[result].y;
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
const _splitTargetsY = (
  targets: Register[],
  classicalRegY: number[],
  registers: RegisterMap,
): number[][] => {
  if (targets.length === 0) return [];

  // Get qIds sorted by ascending y value
  const orderedQIds: number[] = Object.keys(registers).map(Number);
  orderedQIds.sort((a, b) => registers[a].y - registers[b].y);
  const qIdPosition: { [qId: number]: number } = {};
  orderedQIds.forEach((qId, i) => (qIdPosition[qId] = i));

  // Sort targets and classicalRegY by ascending y value
  targets = targets.slice();
  targets.sort((a, b) => {
    const posDiff: number = qIdPosition[a.qubit] - qIdPosition[b.qubit];
    if (posDiff === 0 && a.result != null && b.result != null)
      return a.result - b.result;
    else return posDiff;
  });
  classicalRegY = classicalRegY.slice();
  classicalRegY.sort((a, b) => a - b);

  let prevPos = 0;
  let prevY = 0;

  return targets.reduce((groups: number[][], target: Register) => {
    const y = _getRegY(target, registers);
    const pos = qIdPosition[target.qubit];

    // Split into new group if one of the following holds:
    //      1. First target register
    //      2. Non-adjacent qubit registers
    //      3. There is a classical register between current and previous register
    if (
      groups.length === 0 ||
      pos > prevPos + 1 ||
      (classicalRegY[0] > prevY && classicalRegY[0] < y)
    )
      groups.push([y]);
    else groups[groups.length - 1].push(y);

    prevPos = pos;
    prevY = y;

    // Remove classical registers that are higher than current y
    while (classicalRegY.length > 0 && classicalRegY[0] <= y)
      classicalRegY.shift();

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
const _fillMetadataX = (
  opsMetadata: Metadata[][],
  columnWidths: number[],
): number => {
  let endX: number = startX;

  const colStartX: number[] = columnWidths.map((width) => {
    const x: number = endX;
    endX += width + gatePadding * 2;
    return x;
  });

  opsMetadata.forEach((col, colIndex) =>
    col.forEach((metadata) => {
      const x = colStartX[colIndex];
      switch (metadata.type) {
        case GateType.ClassicalControlled:
        case GateType.Group:
          {
            // Subtract startX offset from nested gates and add offset and padding
            let offset: number = x - startX + groupBoxPadding;
            if (metadata.type === GateType.ClassicalControlled)
              offset += controlBtnOffset;

            // Offset each x coord in children gates
            _offsetChildrenX(metadata.children, offset);

            // We don't use the centre x coord because we only care about the rightmost x for
            // rendering the box around the group of nested gates
            metadata.x = x;
          }
          break;

        default:
          metadata.x = x + columnWidths[colIndex] / 2;
          break;
      }
    }),
  );

  return endX + gatePadding;
};

/**
 * Offset x coords of nested children operations.
 *
 * @param children 2D or 3D array of children metadata.
 * @param offset   x coord offset.
 */
const _offsetChildrenX = (
  children: Metadata[][] | Metadata[][][] | undefined,
  offset: number,
): void => {
  if (children == null) return;
  children.forEach((col) => {
    col.flat().forEach((child) => {
      child.x += offset;
      _offsetChildrenX(child.children, offset);
    });
  });
};

export { processOperations };
