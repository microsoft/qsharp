// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import { box, controlDot } from "./formatters/formatUtils";
import { formatGate } from "./formatters/gateFormatter";
import { toMetadata } from "./panel";
import { Sqore } from "./sqore";
import { findLocation, getHostElems, getWireData } from "./utils";

interface Context {
  container: HTMLElement;
  svg: SVGElement;
  operations: Operation[][];
  wireData: number[];
  renderFn: () => void;
  paddingY: number;
  selectedId: string | null;
  selectedWire: string | null;
}

/**
 * Add draggable elements.
 *
 * @param Container     HTML element for rendering visualization into.
 * @param sqore         Sqore object
 * @param useRefresh    Function to trigger circuit re-rendering
 */
const extensionDraggable = (
  container: HTMLElement,
  sqore: Sqore,
  useRefresh: () => void,
): void => {
  const svg = container.querySelector("svg[id]") as SVGElement;

  const context: Context = {
    container: container,
    svg,
    operations: sqore.circuit.operations,
    wireData: getWireData(container),
    renderFn: useRefresh,
    paddingY: 20,
    selectedId: null,
    selectedWire: null,
  };
  _addStyles(container, getWireData(container));
  _addDataWires(container);
  svg.appendChild(_dropzoneLayer(context));
};

/**
 * Creates a ghost element for dragging operations in the circuit visualization.
 *
 * @param ev The mouse event that triggered the creation of the ghost element.
 * @param container The HTML container element where the ghost element will be appended.
 * @param selectedOperation The operation that is being dragged.
 * @param isControl A boolean indicating if the ghost element is for a control operation.
 */
const createGhostElement = (
  ev: MouseEvent,
  container: HTMLElement,
  selectedOperation: Operation,
  isControl: boolean,
) => {
  const ghost = isControl
    ? controlDot(0, 0)
    : (() => {
        const ghostMetadata = toMetadata(selectedOperation, 0, 0);
        return formatGate(ghostMetadata).cloneNode(true) as SVGElement;
      })();

  // Generate svg element to wrap around ghost element
  const svgElem = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svgElem.append(ghost);

  // Generate div element to wrap around svg element
  const divElem = document.createElement("div");
  divElem.classList.add("ghost");
  divElem.appendChild(svgElem);

  if (container) {
    container.appendChild(divElem);

    // Now that the element is appended to the DOM, get its dimensions
    const ghostRect = ghost.getBoundingClientRect();
    const ghostWidth = ghostRect.width;
    const ghostHeight = ghostRect.height;

    const updateDivLeftTop = (ev: MouseEvent) => {
      divElem.style.left = `${ev.clientX + window.scrollX - ghostWidth / 2}px`;
      divElem.style.top = `${ev.clientY + window.scrollY - ghostHeight / 2}px`;
    };

    updateDivLeftTop(ev);

    container.addEventListener("mousemove", updateDivLeftTop);
  } else {
    console.error("container not found");
  }
};

/**
 * Create a dropzone element that spans the length of the wire.
 *
 * @param circuitSvg The SVG element representing the circuit.
 * @param wireData An array of y values corresponding to the circuit wires.
 * @param wireIndex The index of the wire for which the dropzone is created.
 * @returns The created dropzone SVG element.
 */
const createWireDropzone = (
  circuitSvg: SVGElement,
  wireData: number[],
  wireIndex: number,
): SVGElement => {
  const wireY = wireData[wireIndex];
  const svgWidth = Number(circuitSvg.getAttribute("width"));
  const paddingY = 20;

  const dropzone = box(
    0,
    wireY - paddingY,
    svgWidth,
    paddingY * 2,
    "dropzone-full-wire",
  );
  dropzone.setAttribute("data-dropzone-wire", `${wireIndex}`);

  return dropzone;
};

/**
 * Remove all wire dropzones.
 *
 * @param circuitSvg The SVG element representing the circuit.
 */
const removeAllWireDropzones = (circuitSvg: SVGElement) => {
  const dropzones = circuitSvg.querySelectorAll(".dropzone-full-wire");
  dropzones.forEach((elem) => {
    elem.parentNode?.removeChild(elem);
  });
};

/**
 * Add data-wire to all host elements
 */
const _addDataWires = (container: HTMLElement) => {
  const elems = getHostElems(container);
  elems.forEach((elem) => {
    const { cY } = _center(elem);
    // i.e. cY = 40, wireData returns [40, 100, 140, 180]
    // dataWire will return 0, which is the index of 40 in wireData
    const dataWire = getWireData(container).findIndex((y) => y === cY);
    if (dataWire !== -1) {
      elem.setAttribute("data-wire", `${dataWire}`);
    } else {
      const { y, height } = elem.getBBox();
      const wireData = getWireData(container);
      const groupDataWire = wireData.findIndex(
        (wireY) => wireY > y && wireY < y + height,
      );
      elem.setAttribute("data-wire", `${groupDataWire}`);
    }
  });
};

/**
 * Create a list of wires that element is spanning on
 * i.e. Gate 'Foo' spans on wire 0 (y=40), 1 (y=100), and 2 (y=140)
 *      Function returns [40, 100, 140]
 */
const _wireYs = (elem: SVGGraphicsElement, wireData: number[]): number[] => {
  const { y, height } = elem.getBBox();
  return wireData.filter((wireY) => wireY > y && wireY < y + height);
};

/**
 * Add custom styles specific to this module
 */
const _addStyles = (container: HTMLElement, wireData: number[]): void => {
  const elems = getHostElems(container);
  elems.forEach((elem) => {
    if (_wireYs(elem, wireData).length < 2) elem.style.cursor = "grab";
  });
};

/**
 * Generate an array of wire prefixes from wire data
 */
const _wirePrefixes = (
  wireData: number[],
): { index: number; wireY: number; prefixX: number }[] =>
  wireData.map((wireY, index) => ({ index, wireY, prefixX: 40 }));

/**
 * Find center point of element
 */
const _center = (elem: SVGGraphicsElement): { cX: number; cY: number } => {
  const { x, y, width, height } = elem.getBBox();
  return { cX: x + width / 2, cY: y + height / 2 };
};

/**
 * Create dropzone layer with all dropzones populated
 */
const _dropzoneLayer = (context: Context) => {
  const dropzoneLayer = document.createElementNS(
    "http://www.w3.org/2000/svg",
    "g",
  );
  dropzoneLayer.classList.add("dropzone-layer");
  dropzoneLayer.style.display = "none";

  const { container, svg, wireData, operations, paddingY } = context;
  const elems = getHostElems(container);

  const wirePrefixes = _wirePrefixes(wireData);

  // Sort host elements by its x property
  const sortedElems = Array.from(elems).sort((first, second) => {
    const { x: x1 } = first.getBBox();
    const { x: x2 } = second.getBBox();
    return x1 - x2;
  });

  // Add dropzones for each host elements
  sortedElems.map((elem) => {
    const { cX, cY } = _center(elem);
    const wirePrefix = wirePrefixes.find((item) => item.wireY === cY);

    // Check to prevent group gates creating dropzones between wires
    if (wirePrefix) {
      const { prefixX } = wirePrefix;
      const elemDropzone = box(
        prefixX,
        cY - paddingY,
        cX - prefixX,
        paddingY * 2,
        "dropzone",
      );
      elemDropzone.setAttribute(
        "data-dropzone-location",
        findLocation(elem) || "",
      );
      elemDropzone.setAttribute("data-dropzone-wire", `${wirePrefix.index}`);

      wirePrefix.prefixX = cX;

      dropzoneLayer.appendChild(elemDropzone);
    } else {
      // Let group gates creating dropzones for each wire
      const { x } = elem.getBBox();
      const wireYs = _wireYs(elem, wireData);

      wireYs.map((wireY) => {
        const wirePrefix = wirePrefixes.find((item) => item.wireY === wireY);
        if (wirePrefix) {
          const { prefixX } = wirePrefix;
          const elemDropzone = box(
            prefixX,
            wireY - paddingY,
            x - prefixX,
            paddingY * 2,
            "dropzone",
          );
          elemDropzone.setAttribute(
            "data-dropzone-location",
            findLocation(elem) || "",
          );
          elemDropzone.setAttribute(
            "data-dropzone-wire",
            `${wirePrefix.index}`,
          );

          wirePrefix.prefixX = x;

          dropzoneLayer.appendChild(elemDropzone);
        }
      });
    }
  });

  // Add remaining dropzones to fit max-width of the circuit
  wirePrefixes.map(({ wireY, prefixX }) => {
    const maxWidth = Number(svg.getAttribute("width"));
    const elemDropzone = box(
      prefixX,
      wireY - paddingY,
      maxWidth - prefixX,
      paddingY * 2,
      "dropzone",
    );
    elemDropzone.setAttribute("data-dropzone-location", `${operations.length}`);
    const index = wireData.findIndex((item) => item === wireY);
    elemDropzone.setAttribute("data-dropzone-wire", `${index}`);
    dropzoneLayer.appendChild(elemDropzone);
  });

  return dropzoneLayer;
};

export {
  extensionDraggable,
  createGhostElement,
  createWireDropzone,
  removeAllWireDropzones,
};
