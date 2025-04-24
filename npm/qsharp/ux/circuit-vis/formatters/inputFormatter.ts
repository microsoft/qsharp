// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Qubit } from "../circuit";
import { RegisterType, RegisterMap, RegisterRenderData } from "../register";
import {
  leftPadding,
  startY,
  registerHeight,
  classicalRegHeight,
} from "../constants";
import { group, text } from "./formatUtils";

/**
 * `formatInputs` takes in an array of Qubits and outputs the SVG string of formatted
 * qubit wires and a mapping from register IDs to register rendering data.
 *
 * @param qubits List of declared qubits.
 *
 * @returns returns the SVG string of formatted qubit wires, a mapping from registers
 *          to y coord and total SVG height.
 */
const formatInputs = (
  qubits: Qubit[],
): { qubitWires: SVGElement; registers: RegisterMap; svgHeight: number } => {
  const qubitWires: SVGElement[] = [];
  const registers: RegisterMap = {};

  let currY: number = startY;
  qubits.forEach(({ id, numResults }) => {
    // Add qubit wire to list of qubit wires
    qubitWires.push(_qubitInput(currY, id.toString()));

    // Create qubit register
    registers[id] = { type: RegisterType.Qubit, y: currY };

    // If there are no attached classical registers, increment y by fixed register height
    if (numResults == null || numResults === 0) {
      currY += registerHeight;
      return;
    }

    // Increment current height by classical register height for attached classical registers
    currY += classicalRegHeight;

    // Add classical wires
    registers[id].children = Array.from(Array(numResults), () => {
      const clsReg: RegisterRenderData = {
        type: RegisterType.Classical,
        y: currY,
      };
      currY += classicalRegHeight;
      return clsReg;
    });
  });

  return {
    qubitWires: group(qubitWires, { class: "qubit-input-states" }),
    registers,
    svgHeight: currY,
  };
};

/**
 * Generate the SVG text component for the input qubit register.
 *
 * @param y y coord of input wire to render in SVG.
 *
 * @returns SVG text component for the input register.
 */
const _qubitInput = (y: number, subscript?: string): SVGElement => {
  const el: SVGElement = text("", leftPadding, y, 16);

  // Create the main text node
  const mainText = document.createElementNS(
    "http://www.w3.org/2000/svg",
    "tspan",
  );
  mainText.textContent = "|𝜓";

  // Create the subscript node if provided
  if (subscript) {
    const subscriptText = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "tspan",
    );
    subscriptText.textContent = subscript;
    subscriptText.setAttribute("baseline-shift", "sub");
    subscriptText.setAttribute("font-size", "65%");
    mainText.appendChild(subscriptText);
  }

  // Add the closing part of the text
  const closingText = document.createElementNS(
    "http://www.w3.org/2000/svg",
    "tspan",
  );
  closingText.textContent = "⟩";

  // Append all parts to the main SVG text element
  el.appendChild(mainText);
  el.appendChild(closingText);

  el.setAttribute("text-anchor", "start");
  el.setAttribute("dominant-baseline", "middle");
  return el;
};

export { formatInputs, _qubitInput };
