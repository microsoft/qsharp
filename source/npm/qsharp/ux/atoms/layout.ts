// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TraceData } from "./loader";
import { appendChildren, createSvgElements, setAttributes } from "./utils";

type MachineLayout = {
  cols: number;
  readoutRows: number;
  interactionRows: number;
  storageRows: number;
};

type Location = [number, number, SVGElement?];

type PerStepLayout = Array<{
  qubits: Location[];
  ops: string[];
}>;

const qubitSize = 10;
const zoneSpacing = 10;
const colPadding = 20;
const initialScale = 2.5;
const scaleStep = 0.25;
const speedStep = 0.75;
const zoneBoxCornerRadius = 3;
const doublonCornerRadius = 5;

export function fillQubitLocations(
  rows: number,
  cols: number,
  startRow: number,
): Location[] {
  const qubits: Location[] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      qubits.push([row + startRow, col]);
    }
  }
  return qubits;
}

function parseMove(op: string): { qubit: number; to: Location } | undefined {
  const match = op.match(/move\((\d+), (\d+)\) (\d+)/);
  if (match) {
    const to: Location = [parseInt(match[1]), parseInt(match[2])];
    return { qubit: parseInt(match[3]), to };
  }
  return undefined;
}

function parseGate(
  op: string,
): { gate: string; qubit: number; arg?: string } | undefined {
  const match = op.match(/(\w+)\s*(\(.*\))? (\d+)/);
  if (match) {
    const gate = match[1];
    const qubit = parseInt(match[3]);
    const arg = match[2]
      ? match[2].substring(1, match[2].length - 2)
      : undefined;
    return { gate, qubit, arg };
  }
}

function TraceToPerStepLayout(trace: TraceData): PerStepLayout {
  const perStepLayout: PerStepLayout = [];

  trace.steps.forEach((step, idx) => {
    if (idx == 0) {
      perStepLayout.push({
        qubits: structuredClone(trace.qubits),
        ops: step.ops,
      });
    } else {
      // New locations are the previous locations with the previous step moves applied
      const prevStep = perStepLayout[idx - 1];
      // forEach and map are only invoked for populated elements in sparse arrays
      const prevMoves = prevStep.ops
        .map(parseMove)
        .filter((x) => x != undefined);

      const qubits = structuredClone(prevStep.qubits);
      prevMoves.forEach((move) => {
        qubits[move.qubit] = move.to;
      });
      perStepLayout.push({ qubits, ops: step.ops });
    }
  });
  return perStepLayout;
}

export class Layout {
  container: SVGSVGElement;
  width: number;
  height: number;
  scale: number = initialScale;
  qubits: Location[];
  currentStep = 0;
  trackParent: SVGGElement;
  activeGates: SVGElement[] = [];
  perStepLayout: PerStepLayout;
  showTracks = true;
  stepInterval = 500; // Used for playing and animations

  constructor(
    public layout: MachineLayout,
    trace: TraceData,
  ) {
    if (layout.interactionRows != 1) {
      throw "Only 1 interaction row supported";
    }
    this.perStepLayout = TraceToPerStepLayout(trace);
    this.qubits = structuredClone(this.perStepLayout[0].qubits);

    this.container = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "svg",
    );

    const totalRows =
      layout.readoutRows + layout.interactionRows + layout.storageRows;
    this.height = totalRows * qubitSize + zoneSpacing * 4;
    this.width = layout.cols * qubitSize + colPadding;

    setAttributes(this.container, {
      viewBox: `-5 0 ${this.width} ${this.height}`,
      width: `${this.width * this.scale}px`,
      height: `${this.height * this.scale}px`,
    });

    const readoutOffset = this.getQubitRowOffset(0);
    const interactionOffset = this.getQubitRowOffset(layout.readoutRows);
    const storageOffset = this.getQubitRowOffset(
      layout.readoutRows + layout.interactionRows,
    );
    const colNumOffset = this.getQubitRowOffset(totalRows);

    this.renderZone(readoutOffset, "Readout", layout.readoutRows, layout.cols);
    this.renderDoublons(
      interactionOffset,
      "Interaction",
      layout.cols,
      layout.readoutRows,
    );
    this.renderZone(
      storageOffset,
      "Storage",
      layout.storageRows,
      layout.cols,
      layout.readoutRows + layout.interactionRows,
    );

    // Put the track parent before the qubits, so the qubits render on top
    this.trackParent = createSvgElements("g")[0] as SVGGElement;
    appendChildren(this.container, [this.trackParent]);

    this.renderQubits();
    this.renderColNums(layout.cols, colNumOffset);
  }

  private renderZone(
    offset: number,
    title: string,
    rows: number,
    cols: number,
    firstRowNum = 0,
  ) {
    const g = createSvgElements("g")[0];
    setAttributes(g, {
      transform: `translate(0 ${offset})`,
      class: "qs-atoms-zonebox",
    });
    const rect = createSvgElements("rect")[0];
    setAttributes(rect, {
      x: "0",
      y: "0",
      width: `${cols * qubitSize}`,
      height: `${rows * qubitSize}`,
      rx: `${zoneBoxCornerRadius}`,
    });
    appendChildren(g, [rect]);

    // Draw the lines between the rows
    for (let i = 1; i < rows; i++) {
      const path = createSvgElements("path")[0];
      setAttributes(path, {
        d: `M 0,${i * qubitSize} h${cols * qubitSize}`,
      });
      appendChildren(g, [path]);
    }

    // Number the rows
    for (let i = 0; i < rows; ++i) {
      const rowNum = firstRowNum + i;
      const label = createSvgElements("text")[0];
      setAttributes(label, {
        x: `${cols * qubitSize + 5}`,
        y: `${i * qubitSize + 5}`,
        class: "qs-atoms-label",
      });
      label.textContent = `${rowNum}`;
      appendChildren(g, [label]);
    }

    // Draw the lines between the columns
    for (let i = 1; i < cols; i++) {
      const path = createSvgElements("path")[0];
      setAttributes(path, {
        d: `M ${i * qubitSize},0 v${rows * qubitSize}`,
      });
      appendChildren(g, [path]);
    }

    // Draw the title
    const text = createSvgElements("text")[0];
    setAttributes(text, {
      x: "1",
      y: "-1",
      class: "qs-atoms-zone-text",
    });
    text.textContent = title;

    appendChildren(g, [text]);
    appendChildren(this.container, [g]);
  }

  private renderDoublons(
    offset: number,
    title: string,
    cols: number,
    firstRowNum = 1,
  ) {
    const g = createSvgElements("g")[0];
    setAttributes(g, {
      transform: `translate(0 ${offset})`,
      class: "qs-atoms-zonebox",
    });

    // Draw each doublon
    for (let i = 0; i < cols; i += 2) {
      const rect = createSvgElements("rect")[0];
      setAttributes(rect, {
        x: `${i * qubitSize}`,
        y: "0",
        width: `${qubitSize * 2}`,
        height: `${qubitSize}`,
        rx: `${doublonCornerRadius}`,
      });
      const path = createSvgElements("path")[0];
      setAttributes(path, {
        d: `M ${(i + 1) * qubitSize},0 v${qubitSize}`,
      });
      appendChildren(g, [rect, path]);
    }

    // Number the rows
    for (let i = 0; i < 1 /* only 1 doublon row */; ++i) {
      const rowNum = firstRowNum + i;
      const label = createSvgElements("text")[0];
      setAttributes(label, {
        x: `${cols * qubitSize + 5}`,
        y: `${i * qubitSize + 5}`,
        class: "qs-atoms-label",
      });
      label.textContent = `${rowNum}`;
      appendChildren(g, [label]);
    }

    // Draw the title
    const text = createSvgElements("text")[0];
    setAttributes(text, {
      x: "1",
      y: "-1",
      class: "qs-atoms-zone-text",
    });
    text.textContent = title;

    appendChildren(g, [text]);
    appendChildren(this.container, [g]);
  }

  private renderQubits() {
    const elems = this.qubits.map((location, index) => {
      const [x, y] = this.getQubitCenter(index);

      // Safari has an issue animating multiple attributes concurrently, which we need
      // to do to move the qubit (animate 'cx' and 'cy'), so instead set cx and cy to 0
      // and position the qubit with a transform (see https://stackoverflow.com/a/72022385/1674945)

      const circle = createSvgElements("circle")[0];
      setAttributes(circle, {
        cx: `0`,
        cy: `0`,
        r: `2`,
        class: "qs-atoms-qubit",
      });
      // Animation sets the transform as a style attribute, not an element attribute.
      // Also note, when animating the CSS it requires the 'px' length type (unlike the attribute).
      circle.style.transform = `translate(${x}px, ${y}px)`;
      location[2] = circle;
      return circle;
    });

    appendChildren(this.container, elems);
  }

  private renderColNums(cols: number, offset: number) {
    const g = createSvgElements("g")[0];
    setAttributes(g, {
      transform: `translate(0 ${offset})`,
    });
    // Number the columns
    for (let i = 0; i < cols; ++i) {
      const label = createSvgElements("text")[0];
      setAttributes(label, {
        x: `${i * qubitSize + 5}`,
        y: `5`,
        class: "qs-atoms-label",
      });
      label.textContent = `${i}`;
      appendChildren(g, [label]);
    }
    appendChildren(this.container, [g]);
  }

  renderGateOnQubit(qubit: number, gate: string, arg?: string) {
    if (gate == "RESET") gate = "R";
    const [x, y] = this.getQubitCenter(qubit);

    const gateClass =
      gate === "MZ"
        ? "qs-atoms-gate qs-atoms-gate-mz"
        : gate === "R"
          ? "qs-atoms-gate qs-atoms-gate-reset"
          : "qs-atoms-gate";

    const g = createSvgElements("g")[0];
    setAttributes(g, {
      transform: `translate(${x - qubitSize / 2} ${y - qubitSize / 2})`,
      class: "qs-atoms-gate",
    });

    if (gate === "CZ") {
      // Render the rounded doublon box in a bright color and x--x inside
      const [rect, path, leftDot, rightDot] = createSvgElements(
        "rect",
        "path",
        "circle",
        "circle",
      );
      setAttributes(rect, {
        x: `0`,
        y: "0",
        width: `${qubitSize * 2}`,
        height: `${qubitSize}`,
        rx: `${doublonCornerRadius}`,
        class: "qs-atoms-zonebox qs-atoms-gate-cz",
      });
      // <path d= "M45,5 h10" stroke-width="1.5" stroke="black"/>
      // <circle cx="45" cy="5" r="2" stroke-width="0" fill="#123" />
      // <circle cx="55" cy="5" r="2" stroke-width="0" fill="#123" />
      setAttributes(path, {
        fill: "none",
        stroke: "black",
        "stroke-width": "1.5",
        d: "M5,5 h10",
      });
      setAttributes(leftDot, {
        cx: "5",
        cy: "5",
        r: "2",
        "stroke-width": "0",
        fill: "#123",
      });
      setAttributes(rightDot, {
        cx: "15",
        cy: "5",
        r: "2",
        "stroke-width": "0",
        fill: "#123",
      });
      appendChildren(g, [rect, path, leftDot, rightDot]);
    } else {
      const [rect, text] = createSvgElements("rect", "text");
      setAttributes(rect, {
        x: "0.5",
        y: "0.5",
        width: `${qubitSize - 1}`,
        height: `${qubitSize - 1}`,
        class: gateClass,
      });
      setAttributes(text, {
        x: "5",
        y: arg ? "2.75" : "5",
        class: "qs-atoms-gate-text",
      });
      text.textContent = gate;

      appendChildren(g, [rect, text]);

      if (arg) {
        const argText = createSvgElements("text")[0];
        setAttributes(argText, {
          x: "5",
          y: "7",
          class: "qs-atoms-gate-text qs-atoms-gate-text-small",
          textLength: "8",
        });
        text.classList.add("qs-atoms-gate-text-small");
        argText.textContent = arg;
        appendChildren(g, [argText]);
      }
    }

    appendChildren(this.container, [g]);
    this.activeGates.push(g);
  }

  clearGates() {
    this.activeGates.forEach((gate) => {
      gate.parentElement?.removeChild(gate);
    });
    // TODO: Clear doublons too
    this.activeGates = [];
  }

  zoomIn() {
    this.scale += scaleStep * this.scale;
    setAttributes(this.container, {
      width: `${this.width * this.scale}px`,
      height: `${this.height * this.scale}px`,
    });
  }

  zoomOut() {
    this.scale -= scaleStep * this.scale;
    setAttributes(this.container, {
      width: `${this.width * this.scale}px`,
      height: `${this.height * this.scale}px`,
    });
  }

  faster() {
    this.stepInterval = this.stepInterval * speedStep;
  }

  slower() {
    this.stepInterval = this.stepInterval / speedStep;
  }

  getQubitRowOffset(row: number) {
    if (row < this.layout.readoutRows) {
      return zoneSpacing + row * qubitSize;
    } else if (row < this.layout.readoutRows + this.layout.interactionRows) {
      return 2 * zoneSpacing + row * qubitSize;
    } else {
      return 3 * zoneSpacing + row * qubitSize;
    }
  }

  getLocationCenter(row: number, col: number): [number, number] {
    const x = col * qubitSize + qubitSize / 2;
    const y = this.getQubitRowOffset(row) + qubitSize / 2;
    return [x, y];
  }

  getQubitCenter(qubit: number): [number, number] {
    if (this.qubits[qubit] == undefined) {
      throw "Qubit not found";
    }

    const [row, col] = this.qubits[qubit];
    return this.getLocationCenter(row, col);
  }

  gotoStep(step: number) {
    // Remove prior rendering's gates, trails, or remaining animations
    this.clearGates();
    this.trackParent.replaceChildren();
    this.container.getAnimations({ subtree: true }).forEach((anim) => {
      anim.cancel();
    });

    const forwards = step > this.currentStep;
    this.currentStep = step;

    // When on step 0, just layout the qubits per index 0
    // When on step 1, layout per index 0 then apply the gates/moves per index 0
    // When on step 2, layout per index 1 then apply the gates/moves per index 1
    // etc. until when on step n + 1, layout per index n and apply per index n
    const qubitLocationIndex = step === 0 ? 0 : step - 1;

    // Update all qubit locations
    this.perStepLayout[qubitLocationIndex].qubits.forEach((loc, idx) => {
      const elem = this.qubits[idx][2];
      if (elem === undefined) {
        throw "Invalid qubit index in step";
      }
      this.qubits[idx] = [loc[0], loc[1], elem]; // Update the location

      // Get the offset for the location and move it there
      const [x, y] = this.getQubitCenter(idx);
      elem.style.transform = `translate(${x}px, ${y}px)`;
    });

    // Now apply the ops
    if (step > 0) {
      const duration = forwards ? this.stepInterval / 2 : 0;
      const ops = this.perStepLayout[qubitLocationIndex].ops;
      let trailId = 0;
      ops.forEach((op) => {
        const move = parseMove(op);
        if (move) {
          // Apply the move animation
          const [oldX, oldY] = this.getQubitCenter(move.qubit);
          const [newX, newY] = this.getLocationCenter(move.to[0], move.to[1]);
          const qubit = this.qubits[move.qubit][2];
          if (!qubit) throw "Invalid qubit index";
          if (forwards && this.showTracks) {
            const id = `gradient-${trailId++}`;
            // Draw the path of any qubit movements
            const [gradient, trail] = createSvgElements(
              "linearGradient",
              "line",
            );
            setAttributes(gradient, {
              id,
              gradientUnits: "userSpaceOnUse",
              x1: `${oldX}`,
              y1: `${oldY}`,
              x2: `${newX}`,
              y2: `${newY}`,
            });
            gradient.innerHTML = `<stop offset="0%" stop-color="gray" stop-opacity="0.2"/><stop offset="100%" stop-color="gray" stop-opacity="0.8"/>`;
            setAttributes(trail, {
              x1: `${oldX}`,
              y1: `${oldY}`,
              x2: `${newX}`,
              y2: `${newY}`,
              class: "qs-atoms-qubit-trail",
              style: `stroke-width: 2px; stroke: url(#${id})`,
            });
            appendChildren(this.trackParent, [gradient, trail]);
          }
          qubit
            .animate(
              [
                { transform: `translate(${oldX}px, ${oldY}px)` },
                { transform: `translate(${newX}px, ${newY}px)` },
              ],
              { duration, fill: "forwards", easing: "ease" },
            )
            .finished.then((anim) => {
              anim.commitStyles();
              anim.cancel();
            });
          // TODO: Check if you can/should cancel when scrubbing
        } else {
          // Wasn't a move, so render the gate
          const gate = parseGate(op);
          if (!gate) throw `Invalid gate: ${op}`;
          const arg = gate.arg ? gate.arg.substring(0, 4) : undefined;
          this.renderGateOnQubit(gate.qubit, gate.gate.toUpperCase(), arg);
        }
      });
    }
  }
}
