// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Ket, Measurement, Operation, Unitary } from "./circuit";
import {
  gateHeight,
  horizontalGap,
  minGateWidth,
  verticalGap,
} from "./constants";
import { formatGate } from "./formatters/gateFormatter";
import { GateType, Metadata } from "./metadata";
import { getGateWidth } from "./utils";

/**
 * Create a panel for the circuit visualization.
 * @param container     HTML element for rendering visualization into
 */
const createPanel = (container: HTMLElement): void => {
  if (container.querySelector(".panel") == null) {
    const circuit = container.querySelector("svg[id]");
    if (circuit == null) {
      throw new Error("No circuit found in the container");
    }

    const wrapper = _elem("div", "");
    wrapper.style.display = "block";
    wrapper.style.overflow = "auto";
    wrapper.style.width = "100%";
    wrapper.appendChild(_qubitLineControl());
    container.appendChild(wrapper);
    wrapper.appendChild(circuit);

    const panelElem = _panel();
    container.prepend(panelElem);
    container.style.display = "flex";
    container.style.height = "80vh";
    container.style.width = "95vw";
  }
};

const _qubitLineControl = (): HTMLElement => {
  const qubitLineControlElem = _elem("div", "qubit-line-control");
  _children(qubitLineControlElem, [
    _title("Add/Remove Qubit Lines:"),
    _addQubitLineControl(),
    _removeQubitLineControl(),
  ]);
  return qubitLineControlElem;
};

const _addQubitLineControl = (): HTMLElement => {
  const addQubitLineControlElem = _elem("button", "add-qubit-line");
  addQubitLineControlElem.textContent = "+";
  return addQubitLineControlElem;
};

const _removeQubitLineControl = (): HTMLElement => {
  const removeQubitLineControlElem = _elem("button", "remove-qubit-line");
  removeQubitLineControlElem.textContent = "-";
  return removeQubitLineControlElem;
};

/**
 * Function to produce panel element
 * @param context       Context object to manage extension state
 * @returns             HTML element for panel
 */
const _panel = (): HTMLElement => {
  const panelElem = _elem("div");
  panelElem.className = "panel";
  _children(panelElem, [_createToolbox()]);
  return panelElem;
};

/**
 * Function to produce toolbox element
 * @param context       Context object to manage extension state
 * @returns             HTML element for toolbox
 */
const _createToolbox = (): HTMLElement => {
  // Generate gate elements in a 3xN grid
  let prefixX = 0;
  let prefixY = 0;
  const gateElems = Object.keys(toolboxGateDictionary).map((key, index) => {
    const { width: gateWidth } = toMetadata(toolboxGateDictionary[key], 0, 0);

    // Increment prefixX for every gate, and reset after 2 gates (2 columns)
    if (index % 2 === 0 && index !== 0) {
      prefixX = 0;
      prefixY += gateHeight + verticalGap;
    }

    const gateElem = _gate(
      toolboxGateDictionary,
      key.toString(),
      prefixX,
      prefixY,
    );
    prefixX += gateWidth + horizontalGap;
    return gateElem;
  });

  // Generate svg container to store gate elements
  const svgElem = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svgElem.classList.add("toolbox-panel-svg");
  _childrenSvg(svgElem, gateElems);

  // Generate toolbox panel
  const toolboxElem = _elem("div", "toolbox-panel");
  _children(toolboxElem, [_title("Toolbox")]);
  toolboxElem.appendChild(svgElem);

  return toolboxElem;
};

/**
 * Factory function to produce HTML element
 * @param tag       Tag name
 * @param className Class name
 * @returns         HTML element
 */
const _elem = (tag: string, className?: string): HTMLElement => {
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
const _children = (
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
const _childrenSvg = (
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
const _title = (text: string): HTMLElement => {
  const titleElem = _elem("h2");
  titleElem.className = "title";
  titleElem.textContent = text;
  return titleElem;
};

/**
 * Wrapper to generate metadata based on _opToMetadata with mock registers and limited support
 * @param operation     Operation object
 * @param x             x coordinate at starting point from the left
 * @param y             y coordinate at starting point from the top
 * @returns             Metadata object
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

  if (operation === undefined) return metadata;

  switch (operation.kind) {
    case "unitary": {
      const { gate, controls } = operation;

      if (gate === "SWAP") {
        metadata.type = GateType.Swap;
      } else if (controls && controls.length > 0) {
        metadata.type =
          gate === "X" ? GateType.Cnot : GateType.ControlledUnitary;
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
      break;
    }
    case "measurement":
      metadata.type = GateType.Measure;
      metadata.controlsY = [target];
      break;
    case "ket":
      metadata.type = GateType.Ket;
      metadata.label = operation.gate;
      metadata.targetsY = [[target]];
      break;
  }

  if (operation.args !== undefined && operation.args.length > 0)
    metadata.displayArgs = operation.args[0];

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
const _gate = (
  gateDictionary: GateDictionary,
  type: string,
  x: number,
  y: number,
): SVGElement => {
  const gate = gateDictionary[type];
  if (gate == null) throw new Error(`Gate ${type} not available`);
  const metadata = toMetadata(gate, x, y);
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
 * Function to create a unitary operation
 *
 * @param gate - The name of the gate
 * @returns Unitary operation object
 */
const _makeUnitary = (gate: string): Unitary => {
  return {
    kind: "unitary",
    gate: gate,
    targets: [{ qubit: 0 }],
  };
};

/**
 * Function to create a measurement operation
 *
 * @param gate - The name of the gate
 * @returns Unitary operation object
 */
const _makeMeasurement = (gate: string): Measurement => {
  return {
    kind: "measurement",
    gate: gate,
    qubits: [{ qubit: 0 }],
    results: [{ qubit: 0, result: 0 }],
  };
};

const _makeKet = (gate: string): Ket => {
  return {
    kind: "ket",
    gate: gate,
    targets: [{ qubit: 0 }],
  };
};

/**
 * Object for default gate dictionary
 */
const toolboxGateDictionary: GateDictionary = {
  RX: _makeUnitary("Rx"),
  X: _makeUnitary("X"),
  RY: _makeUnitary("Ry"),
  Y: _makeUnitary("Y"),
  RZ: _makeUnitary("Rz"),
  Z: _makeUnitary("Z"),
  S: _makeUnitary("S"),
  T: _makeUnitary("T"),
  H: _makeUnitary("H"),
  SX: _makeUnitary("SX"),
  Reset: _makeKet("0"),
  Measure: _makeMeasurement("Measure"),
};

toolboxGateDictionary["RX"].params = [{ name: "theta", type: "Double" }];
toolboxGateDictionary["RY"].params = [{ name: "theta", type: "Double" }];
toolboxGateDictionary["RZ"].params = [{ name: "theta", type: "Double" }];

export { createPanel, toolboxGateDictionary, toMetadata };
