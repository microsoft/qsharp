// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Metadata, GateType } from "./metadata";
import {
  minGateWidth,
  labelPadding,
  labelFontSize,
  argsFontSize,
} from "./constants";
import { Operation } from "./circuit";
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
 * Find targets of an operation by recursively walking through all of its children controls and targets.
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
const getGateTargets = (operation: Operation): Register[] | [] => {
  const _recurse = (operation: Operation) => {
    registers.push(...operation.targets);
    if (operation.controls) {
      registers.push(...operation.controls);
      // If there is more children, keep adding more to registers
      if (operation.children) {
        for (const child of operation.children) {
          _recurse(child);
        }
      }
    }
  };

  const registers: Register[] = [];
  if (operation.children == null) return [];

  // Recursively walkthrough all children to populate registers
  for (const child of operation.children) {
    _recurse(child);
  }

  // Extract qIds from array of object
  // i.e. [{qId: 0}, {qId: 1}, {qId: 1}] -> [0, 1, 1]
  const qIds = registers.map((register) => register.qId);
  const uniqueQIds = Array.from(new Set(qIds));

  // Transform array of numbers into array of qId object
  // i.e. [0, 1] -> [{qId: 0}, {qId: 1}]
  return uniqueQIds.map((qId) => ({
    qId,
    type: 0,
  }));
};

/**
 * Split a location string into an array of indexes.
 *
 * Example:
 * "1-2-3" -> [1, 2, 3]
 *
 * @param location The location string to split.
 * @returns An array of indexes.
 */
const locationStringToIndexes = (location: string): number[] => {
  return location !== ""
    ? location.split("-").map((segment) => parseInt(segment))
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
 * @param operations The array of operations to search through.
 * @param location The location string of the operation.
 * @returns The parent operation or null if not found.
 */
const findParentOperation = (
  operations: Operation[],
  location: string | null,
): Operation | null => {
  if (!location) return null;

  const indexes = locationStringToIndexes(location);
  indexes.pop();
  const lastIndex = indexes.pop();

  if (lastIndex == null) return null;

  let parentOperation = operations;
  for (const index of indexes) {
    parentOperation = parentOperation[index].children || parentOperation;
  }
  return parentOperation[lastIndex];
};

/**
 * Find the parent array of an operation based on its location.
 *
 * @param operations The array of operations to search through.
 * @param location The location string of the operation.
 * @returns The parent array of operations or null if not found.
 */
const findParentArray = (
  operations: Operation[],
  location: string | null,
): Operation[] | null => {
  if (!location) return null;

  const indexes = locationStringToIndexes(location);
  indexes.pop(); // The last index refers to the operation itself, remove it so that the last index instead refers to the parent operation

  let parentArray = operations;
  for (const index of indexes) {
    parentArray = parentArray[index].children || parentArray;
  }
  return parentArray;
};

/**
 * Find an operation based on its location.
 *
 * @param operations The array of operations to search through.
 * @param location The location string of the operation.
 * @returns The operation or null if not found.
 */
const findOperation = (
  operations: Operation[],
  location: string | null,
): Operation | null => {
  if (!location) return null;

  const index = locationStringToIndexes(location).pop();
  const operationParent = findParentArray(operations, location);

  if (operationParent == null || index == null) return null;

  return operationParent[index];
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
  getGateTargets,
  locationStringToIndexes,
  getGateLocationString,
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
