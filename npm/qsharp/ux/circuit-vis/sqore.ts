// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { formatInputs } from "./formatters/inputFormatter";
import { formatGates } from "./formatters/gateFormatter";
import { formatRegisters } from "./formatters/registerFormatter";
import { processOperations } from "./process";
import { ConditionalRender, Circuit, Operation } from "./circuit";
import { Metadata, GateType } from "./metadata";
import { StyleConfig, style, STYLES } from "./styles";
import { createUUID } from "./utils";
import { svgNS } from "./constants";
import { extensionDraggable } from "./draggable";
import { extensionEvents } from "./events";
import { extensionPanel, PanelOptions } from "./panel";

/**
 * Contains metadata for visualization.
 */
interface ComposedSqore {
  /** Width of visualization. */
  width: number;
  /** Height of visualization. */
  height: number;
  /** SVG elements the make up the visualization. */
  elements: SVGElement[];
}

/**
 * Defines the mapping of unique location to each operation. Used for enabling
 * interactivity.
 */
type GateRegistry = {
  [location: string]: Operation;
};

type Extension = {
  (container: HTMLElement, sqore: Sqore, useRefresh: () => void): void;
};

/**
 * Entrypoint class for rendering circuit visualizations.
 */
export class Sqore {
  circuit: Circuit;
  style: StyleConfig = {};
  gateRegistry: GateRegistry = {};
  extensions: Extension[] = [];
  renderDepth = 0;

  /**
   * Initializes Sqore object with custom styles.
   *
   * @param circuit Circuit to be visualized.
   * @param style Custom visualization style.
   */
  constructor(circuit: Circuit, style: StyleConfig | string = {}) {
    this.circuit = circuit;
    this.style = this.getStyle(style);
    this.extensions = [];
  }

  /**
   * Render circuit into `container` at the specified layer depth.
   *
   * @param container HTML element for rendering visualization into.
   * @param renderDepth Initial layer depth at which to render gates.
   */
  draw(container: HTMLElement, renderDepth = 0): Sqore {
    // Inject into container
    if (container == null) throw new Error(`Container not provided.`);

    this.renderDepth = renderDepth;
    this.renderCircuit(container);

    return this;
  }

  /**
   * Retrieve style for visualization.
   *
   * @param style Custom style or style name.
   *
   * @returns Custom style.
   */
  private getStyle(style: StyleConfig | string = {}): StyleConfig {
    if (typeof style === "string" || style instanceof String) {
      const styleName: string = style as string;
      // eslint-disable-next-line no-prototype-builtins
      if (!STYLES.hasOwnProperty(styleName)) {
        console.error(`No style ${styleName} found in STYLES.`);
        return {};
      }
      style = STYLES[styleName];
    }
    return style;
  }

  /**
   * Render circuit into `container`.
   *
   * @param container HTML element for rendering visualization into.
   * @param circuit Circuit object to be rendered.
   */
  private renderCircuit(container: HTMLElement, circuit?: Circuit): void {
    // Create copy of circuit to prevent mutation
    const _circuit: Circuit =
      circuit ?? JSON.parse(JSON.stringify(this.circuit));
    const renderDepth = this.renderDepth;

    // Assign unique locations to each operation
    _circuit.operations.forEach((op, i) =>
      this.fillGateRegistry(op, i.toString()),
    );

    // Render operations starting at given depth
    _circuit.operations = this.selectOpsAtDepth(
      _circuit.operations,
      renderDepth,
    );

    // If only one top-level operation, expand automatically:
    if (
      _circuit.operations.length == 1 &&
      _circuit.operations[0].dataAttributes != null &&
      Object.prototype.hasOwnProperty.call(
        _circuit.operations[0].dataAttributes,
        "location",
      )
    ) {
      const location: string =
        _circuit.operations[0].dataAttributes["location"];
      this.expandOperation(_circuit.operations, location);
    }

    // Create visualization components
    const composedSqore: ComposedSqore = this.compose(_circuit);
    const svg: SVGElement = this.generateSvg(composedSqore);
    const previousSvg = container.querySelector("svg[id]");
    if (previousSvg == null) {
      container.appendChild(svg);
    } else {
      container.replaceChild(svg, previousSvg);
    }
    this.addGateClickHandlers(container, _circuit);

    // Run extensions after every render or refresh
    const extensions = this.extensions;
    if (extensions != null) {
      extensions.map((extension) =>
        extension(container, this, () => this.renderCircuit(container)),
      );
    }
  }

  /**
   * Generates the components required for visualization.
   *
   * @param circuit Circuit to be visualized.
   *
   * @returns `ComposedSqore` object containing metadata for visualization.
   */
  private compose(circuit: Circuit): ComposedSqore {
    const add = (acc: Metadata[], gate: Metadata | Metadata[]): void => {
      if (Array.isArray(gate)) {
        gate.forEach((g) => add(acc, g));
      } else {
        acc.push(gate);
        gate.children?.forEach((g) => add(acc, g));
      }
    };

    const flatten = (gates: Metadata[]): Metadata[] => {
      const result: Metadata[] = [];
      add(result, gates);
      return result;
    };

    const { qubits, operations } = circuit;
    const { qubitWires, registers, svgHeight } = formatInputs(qubits);
    const { metadataList, svgWidth } = processOperations(operations, registers);
    const formattedGates: SVGElement = formatGates(metadataList);
    const measureGates: Metadata[] = flatten(metadataList).filter(
      ({ type }) => type === GateType.Measure,
    );
    const formattedRegs: SVGElement = formatRegisters(
      registers,
      measureGates,
      svgWidth,
    );

    const composedSqore: ComposedSqore = {
      width: svgWidth,
      height: svgHeight,
      elements: [qubitWires, formattedRegs, formattedGates],
    };
    return composedSqore;
  }

  /**
   * Generates visualization of `composedSqore` as an SVG.
   *
   * @param composedSqore ComposedSqore to be visualized.
   *
   * @returns SVG representation of circuit visualization.
   */
  private generateSvg(composedSqore: ComposedSqore): SVGElement {
    const { width, height, elements } = composedSqore;
    const uuid: string = createUUID();

    const svg: SVGElement = document.createElementNS(svgNS, "svg");
    svg.setAttribute("id", uuid);
    svg.setAttribute("class", "qviz");
    svg.setAttribute("width", width.toString());
    svg.setAttribute("height", height.toString());
    svg.style.setProperty("max-width", "fit-content");

    // Add styles
    const css = document.createElement("style");
    css.innerHTML = style(this.style);
    css.className = "qviz-style";
    svg.appendChild(css);

    // ToDo: idk why we need to add this style to the document head.
    // For some reason the style node does get added to the svg above
    // until after the user has interacted with the circuit, so we add
    // it to the document head additionally to cover the time before the first interaction.
    const docCss = document.createElement("style");
    docCss.innerHTML = style(this.style);
    docCss.className = "qviz-style";
    document.head.appendChild(docCss);

    // Add body elements
    elements.forEach((element: SVGElement) => svg.appendChild(element));

    return svg;
  }

  /**
   * Depth-first traversal to assign unique location string to `operation`.
   * The operation is assigned the location `location` and its `i`th child is recursively given
   * the location `${location}-${i}`.
   *
   * @param operation Operation to be assigned.
   * @param location: Location to assign to `operation`.
   *
   */
  private fillGateRegistry(operation: Operation, location: string): void {
    if (operation.dataAttributes == null) operation.dataAttributes = {};
    operation.dataAttributes["location"] = location;
    // By default, operations cannot be zoomed-out
    operation.dataAttributes["zoom-out"] = "false";
    this.gateRegistry[location] = operation;
    operation.children?.forEach((childOp, i) => {
      this.fillGateRegistry(childOp, `${location}-${i}`);
      if (childOp.dataAttributes == null) childOp.dataAttributes = {};
      // Children operations can be zoomed out
      childOp.dataAttributes["zoom-out"] = "true";
    });
    // Composite operations can be zoomed in
    operation.dataAttributes["zoom-in"] = (
      operation.children != null
    ).toString();
  }

  /**
   * Pick out operations that are at or below `renderDepth`.
   *
   * @param operations List of circuit operations.
   * @param renderDepth Initial layer depth at which to render gates.
   *
   * @returns List of operations at or below specified depth.
   */
  private selectOpsAtDepth(
    operations: Operation[],
    renderDepth: number,
  ): Operation[] {
    if (renderDepth < 0)
      throw new Error(
        `Invalid renderDepth of ${renderDepth}. Needs to be >= 0.`,
      );
    if (renderDepth === 0) return operations;
    return operations
      .map((op) =>
        op.children != null
          ? this.selectOpsAtDepth(op.children, renderDepth - 1)
          : op,
      )
      .flat();
  }

  /**
   * Add interactive click handlers to circuit HTML elements.
   *
   * @param container HTML element containing visualized circuit.
   * @param circuit Circuit to be visualized.
   *
   */
  private addGateClickHandlers(container: HTMLElement, circuit: Circuit): void {
    this.addClassicalControlHandlers(container);
    this.addZoomHandlers(container, circuit);
  }

  /**
   * Add interactive click handlers for classically-controlled operations.
   *
   * @param container HTML element containing visualized circuit.
   *
   */
  private addClassicalControlHandlers(container: HTMLElement): void {
    container.querySelectorAll(".classically-controlled-btn").forEach((btn) => {
      // Zoom in on clicked gate
      btn.addEventListener("click", (evt: Event) => {
        const textSvg = btn.querySelector("text");
        const group = btn.parentElement;
        if (textSvg == null || group == null) return;

        const currValue = textSvg.firstChild?.nodeValue;
        const zeroGates = group?.querySelector(".gates-zero");
        const oneGates = group?.querySelector(".gates-one");
        switch (currValue) {
          case "?":
            textSvg.childNodes[0].nodeValue = "1";
            group.classList.remove("classically-controlled-unknown");
            group.classList.remove("classically-controlled-zero");
            group.classList.add("classically-controlled-one");
            zeroGates?.classList.add("hidden");
            oneGates?.classList.remove("hidden");
            break;
          case "1":
            textSvg.childNodes[0].nodeValue = "0";
            group.classList.remove("classically-controlled-unknown");
            group.classList.add("classically-controlled-zero");
            group.classList.remove("classically-controlled-one");
            zeroGates?.classList.remove("hidden");
            oneGates?.classList.add("hidden");
            break;
          case "0":
            textSvg.childNodes[0].nodeValue = "?";
            group.classList.add("classically-controlled-unknown");
            group.classList.remove("classically-controlled-zero");
            group.classList.remove("classically-controlled-one");
            zeroGates?.classList.remove("hidden");
            oneGates?.classList.remove("hidden");
            break;
        }
        evt.stopPropagation();
      });
    });
  }

  /**
   * Add interactive click handlers for zoom-in/out functionality.
   *
   * @param container HTML element containing visualized circuit.
   * @param circuit Circuit to be visualized.
   *
   */
  private addZoomHandlers(container: HTMLElement, circuit: Circuit): void {
    container.querySelectorAll(".gate .gate-control").forEach((ctrl) => {
      // Zoom in on clicked gate
      ctrl.addEventListener("click", (ev: Event) => {
        const gateId: string | null | undefined =
          ctrl.parentElement?.getAttribute("data-location");
        if (typeof gateId == "string") {
          if (ctrl.classList.contains("gate-collapse")) {
            this.collapseOperation(circuit.operations, gateId);
          } else if (ctrl.classList.contains("gate-expand")) {
            this.expandOperation(circuit.operations, gateId);
          }
          this.renderCircuit(container, circuit);

          ev.stopPropagation();
        }
      });
    });
  }

  /**
   * Expand selected operation for zoom-in interaction.
   *
   * @param operations List of circuit operations.
   * @param location Location of operation to expand.
   *
   */
  private expandOperation(operations: Operation[], location: string): void {
    operations.forEach((op) => {
      if (op.conditionalRender === ConditionalRender.AsGroup)
        this.expandOperation(op.children || [], location);
      if (op.dataAttributes == null) return op;
      const opId: string = op.dataAttributes["location"];
      if (opId === location && op.children != null) {
        op.conditionalRender = ConditionalRender.AsGroup;
        op.dataAttributes["expanded"] = "true";
      }
    });
  }

  /**
   * Collapse selected operation for zoom-out interaction.
   *
   * @param operations List of circuit operations.
   * @param parentLoc Location of operation to collapse.
   *
   */
  private collapseOperation(operations: Operation[], parentLoc: string): void {
    operations.forEach((op) => {
      if (op.conditionalRender === ConditionalRender.AsGroup)
        this.collapseOperation(op.children || [], parentLoc);
      if (op.dataAttributes == null) return op;
      const opId: string = op.dataAttributes["location"];
      // Collapse parent gate and its children
      if (opId.startsWith(parentLoc)) {
        op.conditionalRender = ConditionalRender.Always;
        delete op.dataAttributes["expanded"];
      }
    });
  }

  public useDraggable(): Sqore {
    this.extensions = [...this.extensions, extensionDraggable];
    return this;
  }

  public usePanel(options?: PanelOptions): Sqore {
    this.extensions = [...this.extensions, extensionPanel(options)];
    return this;
  }

  public useEvents(): Sqore {
    this.extensions = [...this.extensions, extensionEvents];
    return this;
  }

  public useOnCircuitChange(callback: (circuit: Circuit) => void): Sqore {
    const extensionOnCircuitChange = (
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _container: HTMLElement,
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _sqore: Sqore,
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _useRefresh: () => void,
    ) => callback(this.circuit);
    this.extensions = [...this.extensions, extensionOnCircuitChange];
    return this;
  }
}
