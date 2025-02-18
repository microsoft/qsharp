// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import {
  gateHeight,
  horizontalGap,
  minGateWidth,
  minToolboxHeight,
  verticalGap,
} from "./constants";
import { formatGate } from "./formatters/gateFormatter";
import { GateType, Metadata } from "./metadata";
import { Sqore } from "./sqore";
import { getGateWidth } from "./utils";

/**
 * Interface for options provided through usePanel()
 */
interface PanelOptions {
  displaySize?: number;
  gateDictionary?: GateDictionary;
}

/**
 * Entry point to run extensionPanel
 * @param options   User-provided object to customize extensionPanel
 * @returns         Curried function of entry point to run extensionPanel
 */
const extensionPanel =
  (options?: PanelOptions) =>
  /**
   * Curried function of entry point to run extensionPanel
   * @param container     HTML element for rendering visualization into
   * @param sqore         Sqore object
   * @param useRefresh    Function to trigger circuit re-rendering
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  (container: HTMLElement, sqore: Sqore, useRefresh: () => void): void => {
    if (container.querySelector(".panel") == null) {
      const panelElem = panel(options);
      container.prepend(panelElem);
      container.appendChild(qubitLineControl());
    }
  };

const qubitLineControl = (): HTMLElement => {
  const qubitLineControlElem = elem("div", "qubit-line-control");
  children(qubitLineControlElem, [
    title("Add/Remove Qubit Lines:"),
    addQubitLineControl(),
    removeQubitLineControl(),
  ]);
  return qubitLineControlElem;
};

const addQubitLineControl = (): HTMLElement => {
  const addQubitLineControlElem = elem("button", "add-qubit-line");
  addQubitLineControlElem.textContent = "+";
  return addQubitLineControlElem;
};

const removeQubitLineControl = (): HTMLElement => {
  const removeQubitLineControlElem = elem("button", "remove-qubit-line");
  removeQubitLineControlElem.textContent = "-";
  return removeQubitLineControlElem;
};

/**
 * Function to produce panel element
 * @param context       Context object to manage extension state
 * @param options       User-provided object to customize extensionPanel
 * @returns             HTML element for panel
 */
const panel = (options?: PanelOptions): HTMLElement => {
  const panelElem = elem("div");
  panelElem.className = "panel";
  children(panelElem, [createToolbox(options)]);
  return panelElem;
};

/**
 * Function to produce toolbox element
 * @param context       Context object to manage extension state
 * @param options       User-provided object to customize extensionPanel
 * @returns             HTML element for toolbox
 */
const createToolbox = (options?: PanelOptions): HTMLElement => {
  let gateDictionary = defaultGateDictionary;
  let objectKeys = Object.keys(gateDictionary);
  if (options != null) {
    const { displaySize, gateDictionary: customGateDictionary } = options;
    if (displaySize) {
      objectKeys = objectKeys.slice(0, displaySize);
    }
    if (customGateDictionary) {
      gateDictionary = { ...defaultGateDictionary, ...customGateDictionary };
      objectKeys = Object.keys(gateDictionary);
    }
  }

  // Generate gate elements in a 3xN grid
  let prefixX = 0;
  let prefixY = 0;
  const gateElems = objectKeys.map((key) => {
    const { width: gateWidth } = toMetadata(gateDictionary[key], 0, 0);
    if (prefixY + gateHeight + verticalGap > minToolboxHeight) {
      prefixY = 0;
      prefixX += gateWidth + horizontalGap;
    }
    const gateElem = gate(gateDictionary, key.toString(), prefixX, prefixY);
    prefixY += gateHeight + verticalGap;
    return gateElem;
  });

  // Generate svg container to store gate elements
  const svgElem = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svgElem.classList.add("toolbox-panel-svg");
  childrenSvg(svgElem, gateElems);

  // Generate toolbox panel
  const toolboxElem = elem("div", "toolbox-panel");
  children(toolboxElem, [title("Toolbox")]);
  toolboxElem.appendChild(svgElem);

  return toolboxElem;
};

/**
 * Factory function to produce HTML element
 * @param tag       Tag name
 * @param className Class name
 * @returns         HTML element
 */
const elem = (tag: string, className?: string): HTMLElement => {
  const _elem = document.createElement(tag);
  if (className) {
    _elem.className = className;
  }
  return _elem;
};

/**
 * Append all child elements to a parent HTML element
 * @param parentElem    Parent HTML element
 * @param childElems    Array of HTML child elements
 * @returns             Parent HTML element with all children appended
 */
const children = (
  parentElem: HTMLElement,
  childElems: HTMLElement[],
): HTMLElement => {
  childElems.map((elem) => parentElem.appendChild(elem));
  return parentElem;
};

/**
 * Append all child elements to a parent SVG element
 * @param parentElem    Parent SVG element
 * @param childElems    Array of SVG child elements
 * @returns             Parent SVG element with all children appended
 */
const childrenSvg = (
  parentElem: SVGElement,
  childElems: SVGElement[],
): SVGElement => {
  childElems.map((elem) => parentElem.appendChild(elem));
  return parentElem;
};

/**
 * Function to produce title element
 * @param text  Text
 * @returns     Title element
 */
const title = (text: string): HTMLElement => {
  const titleElem = elem("h2");
  titleElem.className = "title";
  titleElem.textContent = text;
  return titleElem;
};

/**
 * Wrapper to generate metadata based on _opToMetadata with mock registers and limited support
 * @param operation     Operation object
 * @param x             x coordinate at starting point from the left
 * @param y             y coordinate at starting point from the top
 * @returns             Metata object
 */
const toMetadata = (
  operation: Operation | undefined,
  x: number,
  y: number,
): Metadata => {
  const target = y + 1 + gateHeight / 2; // offset by 1 for top padding
  const metadata: Metadata = {
    type: GateType.Invalid,
    x: x + 1 + minGateWidth / 2, // offset by 1 for left padding
    controlsY: [],
    targetsY: [target],
    label: "",
    width: -1,
  };

  if (operation == null) return metadata;

  const { gate, displayArgs, isMeasurement, isControlled } = operation;

  // Note: there are a lot of special cases here.
  // It would be good if we could generalize metadata the logic a bit better.
  if (isMeasurement) {
    metadata.type = GateType.Measure;
    metadata.controlsY = [target];
  } else if (gate === "SWAP") {
    metadata.type = GateType.Swap;
  } else if (isControlled) {
    metadata.type = gate === "X" ? GateType.Cnot : GateType.ControlledUnitary;
    metadata.label = gate;
    if (gate !== "X") {
      metadata.targetsY = [[target]];
    }
  } else if (gate === "X") {
    metadata.type = GateType.X;
    metadata.label = gate;
  } else {
    metadata.type = GateType.Unitary;
    metadata.label = gate;
    metadata.targetsY = [[target]];
  }

  if (displayArgs != null) metadata.displayArgs = displayArgs;

  metadata.width = getGateWidth(metadata);
  metadata.x = x + 1 + metadata.width / 2; // offset by 1 for left padding

  return metadata;
};

/**
 * Generate an SVG gate element for the Toolbox panel based on the type of gate.
 * This function retrieves the operation metadata from the gate dictionary,
 * formats the gate, and returns the corresponding SVG element.
 *
 * @param gateDictionary - The dictionary containing gate operations.
 * @param type - The type of gate. Example: 'H' or 'X'.
 * @param x - The x coordinate at the starting point from the left.
 * @param y - The y coordinate at the starting point from the top.
 * @returns The generated SVG element representing the gate.
 * @throws Will throw an error if the gate type is not available in the dictionary.
 */
const gate = (
  gateDictionary: GateDictionary,
  type: string,
  x: number,
  y: number,
): SVGElement => {
  const operation = gateDictionary[type];
  if (operation == null) throw new Error(`Gate ${type} not available`);
  const metadata = toMetadata(operation, x, y);
  metadata.dataAttributes = { type: type };
  const gateElem = formatGate(metadata).cloneNode(true) as SVGElement;
  gateElem.setAttribute("toolbox-item", "true");

  return gateElem;
};

/**
 * Interface for gate dictionary
 */
interface GateDictionary {
  [index: string]: Operation;
}

/**
 * Object for default gate dictionary
 */
const defaultGateDictionary: GateDictionary = {
  RX: {
    gate: "Rx",
    targets: [{ qId: 0, type: 0 }],
  },
  RY: {
    gate: "Ry",
    targets: [{ qId: 0, type: 0 }],
  },
  RZ: {
    gate: "Rz",
    targets: [{ qId: 0, type: 0 }],
  },
  X: {
    gate: "X",
    targets: [{ qId: 0, type: 0 }],
  },
  Y: {
    gate: "Y",
    targets: [{ qId: 0, type: 0 }],
  },
  Z: {
    gate: "Z",
    targets: [{ qId: 0, type: 0 }],
  },
  H: {
    gate: "H",
    targets: [{ qId: 0, type: 0 }],
  },
  S: {
    gate: "S",
    targets: [{ qId: 0, type: 0 }],
  },
  T: {
    gate: "T",
    targets: [{ qId: 0, type: 0 }],
  },
  Measure: {
    gate: "Measure",
    isMeasurement: true,
    controls: [{ qId: 0, type: 0 }],
    targets: [{ qId: 0, type: 1, cId: 0 }],
  },
};

export { extensionPanel, defaultGateDictionary, toMetadata };
export type { PanelOptions };
