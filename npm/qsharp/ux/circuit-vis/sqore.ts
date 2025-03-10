// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { formatInputs } from "./formatters/inputFormatter";
import { formatGates } from "./formatters/gateFormatter";
import { formatRegisters } from "./formatters/registerFormatter";
import { processComponent } from "./process";
import {
  ConditionalRender,
  Circuit,
  CircuitGroup,
  ComponentGrid,
  Column,
  updateToCurrentVersion,
  Component,
} from "./circuit";
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
 * Defines the mapping of unique location to each component. Used for enabling
 * interactivity.
 */
type GateRegistry = {
  [location: string]: Component;
};

type Extension = {
  (container: HTMLElement, sqore: Sqore, useRefresh: () => void): void;
};

/**
 * Entrypoint class for rendering circuit visualizations.
 */
export class Sqore {
  circuitGroup: CircuitGroup;
  circuit: Circuit;
  style: StyleConfig = {};
  gateRegistry: GateRegistry = {};
  extensions: Extension[] = [];
  renderDepth = 0;

  /**
   * Initializes Sqore object with custom styles.
   *
   * @param circuitGroup Group of circuits to be visualized.
   * @param style Custom visualization style.
   */
  constructor(circuitGroup: CircuitGroup, style: StyleConfig | string = {}) {
    const circuits = updateToCurrentVersion(circuitGroup);
    if (
      circuits == null ||
      circuits.circuits == null ||
      circuits.circuits.length === 0
    ) {
      throw new Error(
        `No circuit found in file. Please provide a valid circuit.`,
      );
    }
    this.circuitGroup = circuits;
    // For now we only visualize the first circuit in the group
    this.circuit = circuits.circuits[0];
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

    // Assign unique locations to each component
    _circuit.componentGrid.forEach((col, colIndex) =>
      col.components.forEach((comp, i) =>
        this.fillGateRegistry(comp, `${colIndex},${i}`),
      ),
    );

    // Render components starting at given depth
    _circuit.componentGrid = this.selectCompsAtDepth(
      _circuit.componentGrid,
      renderDepth,
    );

    // If only one top-level component, expand automatically:
    if (
      _circuit.componentGrid.length == 1 &&
      _circuit.componentGrid[0].components.length == 1 &&
      _circuit.componentGrid[0].components[0].dataAttributes != null &&
      Object.prototype.hasOwnProperty.call(
        _circuit.componentGrid[0].components[0].dataAttributes,
        "location",
      )
    ) {
      const location: string =
        _circuit.componentGrid[0].components[0].dataAttributes["location"];
      this.expandComponent(_circuit.componentGrid, location);
    }

    // Create visualization graphics
    const composedSqore: ComposedSqore = this.compose(_circuit);
    const svg: SVGElement = this.generateSvg(composedSqore);
    this.setViewBox(svg);
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
   * Sets the viewBox attribute of the SVG element to enable zooming and panning.
   *
   * @param svg The SVG element to set the viewBox for.
   */
  private setViewBox(svg: SVGElement) {
    // width and height are the true dimensions generated by qviz
    const width = parseInt(svg.getAttribute("width")!);
    const height = svg.getAttribute("height")!;
    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
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
        gate.children?.forEach((col) => col.forEach((g) => add(acc, g)));
      }
    };

    const flatten = (metadata: Metadata[][]): Metadata[] => {
      const result: Metadata[] = [];
      metadata.forEach((col) => col.forEach((g) => add(result, g)));
      return result;
    };

    const { qubits, componentGrid } = circuit;
    const { qubitWires, registers, svgHeight } = formatInputs(qubits);
    const { metadataArray, svgWidth } = processComponent(
      componentGrid,
      registers,
    );
    const formattedGates: SVGElement = formatGates(metadataArray);
    const measureGates: Metadata[] = flatten(metadataArray).filter(
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

    // // ToDo: idk why we need to add this style to the document head.
    // // For some reason the style node does get added to the svg above
    // // until after the user has interacted with the circuit, so we add
    // // it to the document head additionally to cover the time before the first interaction.
    if (!document.head.querySelector("style.qviz-style")) {
      const docCss = document.createElement("style");
      docCss.innerHTML = style(this.style);
      docCss.className = "qviz-style";
      document.head.appendChild(docCss);
    }
    // Add body elements
    elements.forEach((element: SVGElement) => svg.appendChild(element));

    return svg;
  }

  /**
   * Depth-first traversal to assign unique location string to `component`.
   * The component is assigned the location `location` and its `i`th child
   * in its `colIndex` column is recursively given the location
   * `${location}-${colIndex},${i}`.
   *
   * @param component Component to be assigned.
   * @param location: Location to assign to `component`.
   *
   */
  private fillGateRegistry(component: Component, location: string): void {
    if (component.dataAttributes == null) component.dataAttributes = {};
    component.dataAttributes["location"] = location;
    // By default, components cannot be zoomed-out
    component.dataAttributes["zoom-out"] = "false";
    this.gateRegistry[location] = component;
    component.children?.forEach((col, colIndex) =>
      col.components.forEach((childComp, i) => {
        this.fillGateRegistry(childComp, `${location}-${colIndex},${i}`);
        if (childComp.dataAttributes == null) childComp.dataAttributes = {};
        // Children components can be zoomed out
        childComp.dataAttributes["zoom-out"] = "true";
      }),
    );
    // Composite components can be zoomed in
    component.dataAttributes["zoom-in"] = (
      component.children != null
    ).toString();
  }

  /**
   * Pick out components that are at or below `renderDepth`.
   *
   * @param componentGrid Circuit components.
   * @param renderDepth Initial layer depth at which to render gates.
   *
   * @returns Grid of components at or below specified depth.
   */
  private selectCompsAtDepth(
    componentGrid: ComponentGrid,
    renderDepth: number,
  ): ComponentGrid {
    if (renderDepth < 0)
      throw new Error(
        `Invalid renderDepth of ${renderDepth}. Needs to be >= 0.`,
      );
    if (renderDepth === 0) return componentGrid;
    const selectedComps: ComponentGrid = [];
    componentGrid.forEach((col) => {
      const selectedCol: Component[] = [];
      const extraCols: Column[] = [];
      col.components.forEach((comp) => {
        if (comp.children != null) {
          const selectedChildren = this.selectCompsAtDepth(
            comp.children,
            renderDepth - 1,
          );
          if (selectedChildren.length > 0) {
            selectedCol.push(...selectedChildren[0].components);
            selectedChildren.slice(1).forEach((col, colIndex) => {
              if (extraCols[colIndex] == null) extraCols[colIndex] = col;
              // NOTE: I'm unsure if this is a safe way to combine column arrays
              else extraCols[colIndex].components.push(...col.components);
            });
          }
        } else {
          selectedCol.push(comp);
        }
        selectedComps.push({ components: selectedCol });
        if (extraCols.length > 0) {
          selectedComps.push(...extraCols);
        }
      });
    });
    return selectedComps;
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
   * Add interactive click handlers for classically-controlled components.
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
            this.collapseComponent(circuit.componentGrid, gateId);
          } else if (ctrl.classList.contains("gate-expand")) {
            this.expandComponent(circuit.componentGrid, gateId);
          }
          this.renderCircuit(container, circuit);

          ev.stopPropagation();
        }
      });
    });
  }

  /**
   * Expand selected component for zoom-in interaction.
   *
   * @param componentGrid Grid of circuit components.
   * @param location Location of component to expand.
   *
   */
  private expandComponent(
    componentGrid: ComponentGrid,
    location: string,
  ): void {
    componentGrid.forEach((col) =>
      col.components.forEach((comp) => {
        if (comp.conditionalRender === ConditionalRender.AsGroup)
          this.expandComponent(comp.children || [], location);
        if (comp.dataAttributes == null) return comp;
        const opId: string = comp.dataAttributes["location"];
        if (opId === location && comp.children != null) {
          comp.conditionalRender = ConditionalRender.AsGroup;
          comp.dataAttributes["expanded"] = "true";
        }
      }),
    );
  }

  /**
   * Collapse selected component for zoom-out interaction.
   *
   * @param componentGrid Grid of circuit components.
   * @param parentLoc Location of component to collapse.
   *
   */
  private collapseComponent(
    componentGrid: ComponentGrid,
    parentLoc: string,
  ): void {
    componentGrid.forEach((col) =>
      col.components.forEach((comp) => {
        if (comp.conditionalRender === ConditionalRender.AsGroup)
          this.collapseComponent(comp.children || [], parentLoc);
        if (comp.dataAttributes == null) return comp;
        const opId: string = comp.dataAttributes["location"];
        // Collapse parent gate and its children
        if (opId.startsWith(parentLoc)) {
          comp.conditionalRender = ConditionalRender.Always;
          delete comp.dataAttributes["expanded"];
        }
      }),
    );
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

  public useOnCircuitChange(callback: (fileData: CircuitGroup) => void): Sqore {
    const extensionOnCircuitChange = (
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _container: HTMLElement,
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _sqore: Sqore,
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      _useRefresh: () => void,
    ) => callback(this.minimizeCircuits(this.circuitGroup));
    this.extensions = [...this.extensions, extensionOnCircuitChange];
    return this;
  }

  // Note: I'm unsure that this is the right place in code to do this.

  // Minimize the circuits in a circuit group to remove dataAttributes
  minimizeCircuits(circuitGroup: CircuitGroup): CircuitGroup {
    const minimizedCircuits: CircuitGroup = JSON.parse(
      JSON.stringify(circuitGroup),
    );
    minimizedCircuits.circuits.forEach((circuit) => {
      circuit.componentGrid.forEach((col) => {
        col.components.forEach(this.minimizeComponent);
      });
    });
    return minimizedCircuits;
  }

  // Minimize the component to remove dataAttributes
  minimizeComponent = (component: Component): void => {
    if (component.children !== undefined) {
      component.children.forEach((col) =>
        col.components.forEach(this.minimizeComponent),
      );
    }
    component.dataAttributes = undefined;
  };
}
