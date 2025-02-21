// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import { gatePadding, minGateWidth, startX } from "./constants";
import { box, controlDot } from "./formatters/formatUtils";
import { formatGate } from "./formatters/gateFormatter";
import { toMetadata } from "./panel";
import { getMinMaxRegIdx } from "./process";
import { Sqore } from "./sqore";
import {
  findLocation,
  getHostElems,
  getWireData,
  locationStringToIndexes,
} from "./utils";

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

  const { container, operations, wireData, paddingY } = context;
  if (wireData.length === 0) return dropzoneLayer; // Return early if there are no wires
  const elems = getHostElems(container);

  // Get the widths of each column based on the elements in the column
  const colWidths = elems.reduce(
    (acc, elem) => {
      const location = findLocation(elem);
      if (!location) return acc;
      const indexes = locationStringToIndexes(location);
      // NOTE: for now, we are just going to consider the widths of top-level gates
      if (indexes.length != 1) return acc;
      const [colIndex] = indexes[0];

      if (!acc[colIndex]) {
        acc[colIndex] = Math.max(minGateWidth, elem.getBBox().width);
      } else {
        acc[colIndex] = Math.max(acc[colIndex], elem.getBBox().width);
      }
      return acc;
    },
    {} as Record<number, number>,
  );

  // Sort colWidths by colIndex
  const sortedColWidths = Object.entries(colWidths)
    .sort(([colIndexA], [colIndexB]) => Number(colIndexA) - Number(colIndexB))
    .map(([colIndex, colWidth]) => [Number(colIndex), colWidth]);

  let xOffset = startX / 2;

  // Create dropzones for each intersection of columns and wires
  sortedColWidths.forEach(([colIndex, colWidth]) => {
    const columnOps = operations[colIndex];
    let wireIndex = 0;

    columnOps.forEach((op, opIndex) => {
      const [minTarget, maxTarget] = getMinMaxRegIdx(op, wireData.length);
      // Add dropzones before the first target
      while (wireIndex <= maxTarget) {
        const wireY = wireData[wireIndex];
        const dropzone = box(
          xOffset,
          wireY - paddingY,
          colWidth + gatePadding * 2,
          paddingY * 2,
          "dropzone",
        );
        dropzone.setAttribute(
          "data-dropzone-location",
          `${colIndex},${opIndex}`,
        );
        const shouldPushOps = wireIndex >= minTarget;
        dropzone.setAttribute("data-dropzone-wire", `${wireIndex}`);
        dropzone.setAttribute("data-dropzone-push", `${shouldPushOps}`);
        dropzoneLayer.appendChild(dropzone);
        wireIndex++;
      }
    });

    // Add dropzones after the last target
    while (wireIndex < wireData.length) {
      const wireY = wireData[wireIndex];
      const dropzone = box(
        xOffset,
        wireY - paddingY,
        colWidth + gatePadding * 2,
        paddingY * 2,
        "dropzone",
      );
      dropzone.setAttribute(
        "data-dropzone-location",
        `${colIndex},${columnOps.length}`,
      );
      dropzone.setAttribute("data-dropzone-wire", `${wireIndex}`);
      dropzone.setAttribute("data-dropzone-push", "false");
      dropzoneLayer.appendChild(dropzone);
      wireIndex++;
    }
    xOffset += colWidth + gatePadding * 2;
  });

  // This assumes column indexes are continuous
  const endColIndex = sortedColWidths.length;

  // Add remaining dropzones to fit max-width of the circuit
  wireData.forEach((wireY, wireIndex) => {
    const dropzone = box(
      xOffset,
      wireY - paddingY,
      minGateWidth / 2 + gatePadding * 2,
      paddingY * 2,
      "dropzone",
    );
    dropzone.setAttribute("data-dropzone-location", `${endColIndex},0`);
    dropzone.setAttribute("data-dropzone-push", "false");
    dropzone.setAttribute("data-dropzone-wire", `${wireIndex}`);
    dropzoneLayer.appendChild(dropzone);
  });

  return dropzoneLayer;
};

export {
  extensionDraggable,
  createGhostElement,
  createWireDropzone,
  removeAllWireDropzones,
};
