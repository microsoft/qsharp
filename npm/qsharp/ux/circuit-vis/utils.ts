// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Metadata, GateType } from "./metadata";
import {
  minGateWidth,
  labelPadding,
  labelFontSize,
  argsFontSize,
} from "./constants";
import { ComponentGrid, Operation } from "./circuit";
import { Register } from "./register";

/**
 * Generate a UUID using `Math.random`.
 * Note: this implementation came from https://stackoverflow.com/questions/105034/how-to-create-guid-uuid
 * and is not cryptographically secure but works for our use case.
 *
 * @returns UUID string.
 */
const createUUID = (): string =>
  "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0,
      v = c == "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });

/**
 * Calculate the width of a gate, given its metadata.
 *
 * @param metadata Metadata of a given gate.
 *
 * @returns Width of given gate (in pixels).
 */
const getGateWidth = ({
  type,
  label,
  displayArgs,
  width,
}: Metadata): number => {
  if (width > 0) return width;

  switch (type) {
    case GateType.Measure:
    case GateType.Cnot:
    case GateType.Swap:
      return minGateWidth;
    default: {
      const labelWidth = _getStringWidth(label);
      const argsWidth =
        displayArgs != null ? _getStringWidth(displayArgs, argsFontSize) : 0;
      const textWidth = Math.max(labelWidth, argsWidth) + labelPadding * 2;
      return Math.max(minGateWidth, textWidth);
    }
  }
};

/**
 * Get the width of a string with font-size `fontSize` and font-family Arial.
 *
 * @param text     Input string.
 * @param fontSize Font size of `text`.
 *
 * @returns Pixel width of given string.
 */
const _getStringWidth = (
  text: string,
  fontSize: number = labelFontSize,
): number => {
  const canvas: HTMLCanvasElement = document.createElement("canvas");
  const context: CanvasRenderingContext2D | null = canvas.getContext("2d");
  if (context == null) throw new Error("Null canvas");

  context.font = `${fontSize}px Arial`;
  const metrics: TextMetrics = context.measureText(text);
  return metrics.width;
};

/**
 * Find targets of an operation's children by recursively walking
 * through all of its children's controls and targets.
 * Note that this intensionally ignores the direct targets of the
 * operation itself.
 *
 * Example:
 * Gate Foo contains gate H and gate RX.
 * qIds of Gate H is 1
 * qIds of Gate RX are 1, 2
 * This should return [{qId: 1}, {qId: 2}]
 *
 * @param operation The operation to find targets for.
 * @returns An array of registers with unique qIds.
 */
const getChildTargets = (operation: Operation): Register[] | [] => {
  const _recurse = (operation: Operation) => {
    if (operation.kind === "measurement") {
      registers.push(...operation.qubits);
      registers.push(...operation.results);
    } else if (operation.kind === "unitary") {
      registers.push(...operation.targets);
      if (operation.controls) {
        registers.push(...operation.controls);
      }
    }

    // If there is more children, keep adding more to registers
    if (operation.children) {
      operation.children.forEach((col) =>
        col.components.forEach((child) => {
          _recurse(child);
        }),
      );
    }
  };

  const registers: Register[] = [];
  if (operation.children == null) return [];

  // Recursively walkthrough all children to populate registers
  operation.children.forEach((col) =>
    col.components.forEach((child) => {
      _recurse(child);
    }),
  );

  // Extract qIds from array of object
  // i.e. [{qId: 0}, {qId: 1}, {qId: 1}] -> [0, 1, 1]
  const qIds = registers.map((register) => register.qubit);
  const uniqueQIds = Array.from(new Set(qIds));

  // Transform array of numbers into array of qId object
  // i.e. [0, 1] -> [{qId: 0}, {qId: 1}]
  return uniqueQIds.map((qId) => ({ qubit: qId }));
};

/**
 * Split a location string into an array of index tuples.
 *
 * Example:
 * "0,1-0,2-2,3" -> [[0,1], [0,2], [2,3]]
 *
 * @param location The location string to split.
 * @returns An array of indexes.
 */
const locationStringToIndexes = (location: string): [number, number][] => {
  return location !== ""
    ? location.split("-").map((segment) => {
        const coords = segment.split(",");
        if (coords.length !== 2) throw new Error("Invalid location");
        return [parseInt(coords[0]), parseInt(coords[1])];
      })
    : [];
};

/**
 * Gets the location of an operation, if it has one.
 *
 * @param operation The operation to get the location for.
 * @returns The location string of the operation, or null if it doesn't have one.
 */
const getGateLocationString = (operation: Operation): string | null => {
  if (operation.dataAttributes == null) return null;
  return operation.dataAttributes["location"];
};

/**
 * Get the label from a ket string.
 *
 * @param ket The ket string to extract the label from.
 * @returns The label extracted from the ket string.
 */
const getKetLabel = (ket: string): string => {
  // Check that the ket conforms to the format |{label}> or |{label}⟩
  const ketRegex = /^\|([^\s〉⟩〉>]+)(?:[〉⟩〉>])$/;

  // Match the ket string against the regex
  const match = ket.match(ketRegex);

  // If valid, return the inner label (captured group 1), otherwise return an empty string
  return match ? match[1] : "";
};

/**
 * Converts a list of operations into a 2D grid of operations in col-row format.
 * Operations will be left-justified as much as possible in the resulting grid.
 * Children operations are recursively converted into a grid.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns A 2D array of operations.
 */
const operationListToGrid = (
  operations: Operation[],
  numQubits: number,
): ComponentGrid => {
  operations.forEach((op) => {
    // The children data structure is a grid, so checking if it is
    // length 1 is actually checking if it has a single column,
    // or in other words, we are checking if its children are in a single list.
    // If the operation has children in a single list, it needs to be converted to a grid.
    // If it was already converted to a grid, but the grid was still a single list,
    // then doing it again won't effect anything.
    if (op.children && op.children.length == 1) {
      op.children = operationListToGrid(op.children[0].components, numQubits);
    }
  });

  return _removePadding(_operationListToPaddedArray(operations, numQubits)).map(
    (col) => ({
      components: col,
    }),
  );
};

/**
 * Converts a list of operations into a padded 2D array of operations.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns A 2D array of operations padded with `null`s.
 */
const _operationListToPaddedArray = (
  operations: Operation[],
  numQubits: number,
): (Operation | null)[][] => {
  if (operations.length === 0) return [];

  // Group operations based on registers
  const groupedOps: number[][] = _groupOperations(operations, numQubits);

  // Align operations on multiple registers
  const alignedOps: (number | null)[][] = _transformToColRow(
    _alignOps(groupedOps),
  );

  const operationArray: (Operation | null)[][] = alignedOps.map((col) =>
    col.map((opIdx) => {
      if (opIdx == null) return null;
      return operations[opIdx];
    }),
  );

  return operationArray;
};

/**
 * Removes padding (`null` values) from a 2D array of operations.
 *
 * @param operations 2D array of operations padded with `null`s.
 *
 * @returns A 2D array of operations without `null` values.
 */
const _removePadding = (operations: (Operation | null)[][]): Operation[][] => {
  return operations.map((col) => col.filter((op) => op != null));
};

/**
 * Transforms a row-col 2D array into an equivalent col-row 2D array.
 *
 * @param alignedOps 2D array of operations in row-col format.
 *
 * @returns 2D array of operations in col-row format.
 */
const _transformToColRow = (
  alignedOps: (number | null)[][],
): (number | null)[][] => {
  if (alignedOps.length === 0) return [];

  const numRows = alignedOps.length;
  const numCols = Math.max(...alignedOps.map((row) => row.length));

  const colRowArray: (number | null)[][] = Array.from({ length: numCols }, () =>
    Array(numRows).fill(null),
  );

  for (let row = 0; row < numRows; row++) {
    for (let col = 0; col < alignedOps[row].length; col++) {
      colRowArray[col][row] = alignedOps[row][col];
    }
  }

  return colRowArray;
};

/**
 * Get the minimum and maximum register indices for a given operation.
 *
 * @param operation The operation for which to get the register indices.
 * @param numQubits The number of qubits in the circuit.
 * @returns A tuple containing the minimum and maximum register indices.
 */
const getMinMaxRegIdx = (
  operation: Operation,
  numQubits: number,
): [number, number] => {
  const { targets, controls } =
    operation.kind === "measurement"
      ? { targets: operation.results, controls: operation.qubits }
      : { targets: operation.targets, controls: operation.controls };
  const ctrls: Register[] = controls || [];
  const qRegs: Register[] = [...ctrls, ...targets].filter(
    ({ result }) => result === undefined,
  );
  const qRegIdxList: number[] = qRegs.map(({ qubit }) => qubit);
  const clsControls: Register[] = ctrls.filter(
    ({ result }) => result !== undefined,
  );
  const isClassicallyControlled: boolean = clsControls.length > 0;
  if (!isClassicallyControlled && qRegs.length === 0) return [-1, -1];
  // If operation is classically-controlled, pad all qubit registers. Otherwise, only pad
  // the contiguous range of registers that it covers.
  const minRegIdx: number = isClassicallyControlled
    ? 0
    : Math.min(...qRegIdxList);
  const maxRegIdx: number = isClassicallyControlled
    ? numQubits - 1
    : Math.max(...qRegIdxList);

  return [minRegIdx, maxRegIdx];
};

/**
 * Group gates provided by operations into their respective registers.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns 2D array of indices where `groupedOps[i][j]` is the index of the operations
 *          at register `i` and column `j` (not yet aligned/padded).
 */
const _groupOperations = (
  operations: Operation[],
  numQubits: number,
): number[][] => {
  const groupedOps: number[][] = Array.from(
    Array(numQubits),
    () => new Array(0),
  );
  operations.forEach((operation, instrIdx) => {
    const [minRegIdx, maxRegIdx] = getMinMaxRegIdx(operation, numQubits);
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
  const paddedOps: (number | null)[][] = ops.map((regOps) => [...regOps]);
  while (col < maxNumOps) {
    for (let regIdx = 0; regIdx < paddedOps.length; regIdx++) {
      const reg: (number | null)[] = paddedOps[regIdx];
      if (reg.length <= col) continue;

      // Should never be null (nulls are only padded to previous columns)
      const opIdx: number | null = reg[col];

      // Get position of gate
      const targetsPos: number[] = paddedOps.map((regOps) =>
        regOps.indexOf(opIdx),
      );
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

/**********************
 *  Finder Functions  *
 **********************/

/**
 * Find the surrounding gate element of a host element.
 *
 * @param hostElem The SVG element representing the host element.
 * @returns The surrounding gate element or null if not found.
 */
const findGateElem = (hostElem: SVGElement): SVGElement | null => {
  return hostElem.closest<SVGElement>("[data-location]");
};

/**
 * Find the location of the gate surrounding a host element.
 *
 * @param hostElem The SVG element representing the host element.
 * @returns The location string of the surrounding gate or null if not found.
 */
const findLocation = (hostElem: SVGElement) => {
  const gateElem = findGateElem(hostElem);
  return gateElem != null ? gateElem.getAttribute("data-location") : null;
};

/**
 * Find the parent operation of the operation specified by location.
 *
 * @param componentGrid The grid of components to search through.
 * @param location The location string of the operation.
 * @returns The parent operation or null if not found.
 */
const findParentOperation = (
  componentGrid: ComponentGrid,
  location: string | null,
): Operation | null => {
  if (!location) return null;

  const indexes = locationStringToIndexes(location);
  indexes.pop();
  const lastIndex = indexes.pop();

  if (lastIndex == null) return null;

  let parentOperation = componentGrid;
  for (const index of indexes) {
    parentOperation =
      parentOperation[index[0]].components[index[1]].children ||
      parentOperation;
  }
  return parentOperation[lastIndex[0]].components[lastIndex[1]];
};

/**
 * Find the parent component grid of an operation based on its location.
 *
 * @param componentGrid The grid of components to search through.
 * @param location The location string of the operation.
 * @returns The parent grid of components or null if not found.
 */
const findParentArray = (
  componentGrid: ComponentGrid,
  location: string | null,
): ComponentGrid | null => {
  if (!location) return null;

  const indexes = locationStringToIndexes(location);
  indexes.pop(); // The last index refers to the operation itself, remove it so that the last index instead refers to the parent operation

  let parentArray = componentGrid;
  for (const index of indexes) {
    parentArray =
      parentArray[index[0]].components[index[1]].children || parentArray;
  }
  return parentArray;
};

/**
 * Find an operation based on its location.
 *
 * @param componentGrid The grid of components to search through.
 * @param location The location string of the operation.
 * @returns The operation or null if not found.
 */
const findOperation = (
  componentGrid: ComponentGrid,
  location: string | null,
): Operation | null => {
  if (!location) return null;

  const index = locationStringToIndexes(location).pop();
  const operationParent = findParentArray(componentGrid, location);

  if (operationParent == null || index == null) return null;

  return operationParent[index[0]].components[index[1]];
};

/**********************
 *  Getter Functions  *
 **********************/

/**
 * Get list of y values based on circuit wires.
 *
 * @param container The HTML container element containing the circuit visualization.
 * @returns An array of y values corresponding to the circuit wires.
 */
const getWireData = (container: HTMLElement): number[] => {
  // elems include qubit wires and lines of measure gates
  const elems = container.querySelectorAll<SVGGElement>(
    "svg[id] > g:nth-child(3) > g",
  );
  // filter out <g> elements having more than 2 elements because
  // qubit wires contain only 2 elements: <line> and <text>
  // lines of measure gates contain 4 <line> elements
  const wireElems = Array.from(elems).filter(
    (elem) => elem.childElementCount < 3,
  );
  const wireData = wireElems.map((wireElem) => {
    const lineElem = wireElem.children[0] as SVGLineElement;
    return Number(lineElem.getAttribute("y1"));
  });
  return wireData;
};

/**
 * Get list of toolbox items.
 *
 * @param container The HTML container element containing the toolbox items.
 * @returns An array of SVG graphics elements representing the toolbox items.
 */
const getToolboxElems = (container: HTMLElement): SVGGraphicsElement[] => {
  return Array.from(
    container.querySelectorAll<SVGGraphicsElement>("[toolbox-item]"),
  );
};

/**
 * Get list of host elements that dropzones can be attached to.
 *
 * @param container The HTML container element containing the circuit visualization.
 * @returns An array of SVG graphics elements representing the host elements.
 */
const getHostElems = (container: HTMLElement): SVGGraphicsElement[] => {
  const circuitSvg = container.querySelector("svg[id]");
  return circuitSvg != null
    ? Array.from(
        circuitSvg.querySelectorAll<SVGGraphicsElement>(
          '[class^="gate-"]:not(.gate-control, .gate-swap), .control-dot, .oplus, .cross',
        ),
      )
    : [];
};

/**
 * Get list of gate elements from the circuit, but not the toolbox.
 *
 * @param container The HTML container element containing the circuit visualization.
 * @returns An array of SVG graphics elements representing the gate elements.
 */
const getGateElems = (container: HTMLElement): SVGGraphicsElement[] => {
  const circuitSvg = container.querySelector("svg[id]");
  return circuitSvg != null
    ? Array.from(circuitSvg.querySelectorAll<SVGGraphicsElement>(".gate"))
    : [];
};

export {
  createUUID,
  getGateWidth,
  getChildTargets,
  locationStringToIndexes,
  getGateLocationString,
  getKetLabel,
  operationListToGrid,
  getMinMaxRegIdx,
  findGateElem,
  findLocation,
  findParentOperation,
  findParentArray,
  findOperation,
  getWireData,
  getToolboxElems,
  getHostElems,
  getGateElems,
};
