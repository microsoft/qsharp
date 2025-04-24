// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { getMinMaxRegIdx } from "../../src/utils";
import { ComponentGrid, Operation } from "./circuit";
import { gatePadding, minGateWidth, startX } from "./constants";
import { box, controlDot } from "./formatters/formatUtils";
import { formatGate } from "./formatters/gateFormatter";
import { toMetadata } from "./panel";
import { Sqore } from "./sqore";
import {
  findLocation,
  getHostElems,
  getToolboxElems,
  getWireData,
  locationStringToIndexes,
} from "./utils";

interface Context {
  container: HTMLElement;
  operationGrid: ComponentGrid;
  wireData: number[];
}

/**
 * Create dragzones elements for dragging on circuit.
 *
 * @param container     HTML element for rendering visualization into
 * @param sqore         Sqore object
 */
const createDragzones = (container: HTMLElement, sqore: Sqore): void => {
  const svg = container.querySelector("svg[id]") as SVGElement;

  const context: Context = {
    container,
    operationGrid: sqore.circuit.componentGrid,
    wireData: getWireData(container),
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
    ? controlDot(20, 20)
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
  divElem.style.position = "fixed";

  if (container) {
    container.appendChild(divElem);

    // Now that the element is appended to the DOM, get its dimensions
    const [ghostWidth, ghostHeight] = isControl
      ? [40, 40]
      : (() => {
          const ghostRect = ghost.getBoundingClientRect();
          return [ghostRect.width, ghostRect.height];
        })();

    const updateDivLeftTop = (ev: MouseEvent) => {
      divElem.style.left = `${ev.clientX - ghostWidth / 2}px`;
      divElem.style.top = `${ev.clientY - ghostHeight / 2}px`;
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

  const toolBoxElems = getToolboxElems(container);
  toolBoxElems.forEach((elem) => {
    elem.style.cursor = "grab";
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

  const { container, operationGrid, wireData } = context;
  if (wireData.length === 0) return dropzoneLayer; // Return early if there are no wires

  const colArray = getColumnOffsetsAndWidths(container);

  // Create dropzones for each intersection of columns and wires
  for (let colIndex = 0; colIndex < colArray.length; colIndex++) {
    const columnOps = operationGrid[colIndex];
    let wireIndex = 0;

    const makeBox = (opIndex: number, interColumn: boolean) =>
      makeDropzoneBox(
        colIndex,
        opIndex,
        colArray,
        wireData,
        wireIndex,
        interColumn,
      );

    columnOps.components.forEach((op, opIndex) => {
      const [minTarget, maxTarget] = getMinMaxRegIdx(op, wireData.length);
      // Add dropzones before the first target
      while (wireIndex <= maxTarget) {
        dropzoneLayer.appendChild(makeBox(opIndex, true));
        // We don't want to make a central zone if the spot is occupied by a gate or its connecting lines
        if (wireIndex < minTarget) {
          dropzoneLayer.appendChild(makeBox(opIndex, false));
        }

        wireIndex++;
      }
    });

    // Add dropzones after the last target
    while (wireIndex < wireData.length) {
      dropzoneLayer.appendChild(makeBox(columnOps.components.length, true));
      dropzoneLayer.appendChild(makeBox(columnOps.components.length, false));

      wireIndex++;
    }
  }

  // This assumes column indexes are continuous
  const endColIndex = colArray.length;

  // Add remaining dropzones to allow users to add gates to the end of the circuit
  for (let wireIndex = 0; wireIndex < wireData.length; wireIndex++) {
    const dropzone = makeDropzoneBox(
      endColIndex,
      0,
      colArray,
      wireData,
      wireIndex,
      true,
    );
    // Note: the last column should have the shape of an inter-column dropzone, but
    // we don't want to attach the inter-column logic to it.
    dropzone.setAttribute("data-dropzone-inter-column", "false");
    dropzoneLayer.appendChild(dropzone);
  }

  return dropzoneLayer;
};

/**
 * Computes a sorted array of { xOffset, colWidth } for each column index.
 * The array index corresponds to the column index.
 *
 * @param container The circuit container element.
 * @returns Array where arr[colIndex] = { xOffset, colWidth }
 */
const getColumnOffsetsAndWidths = (
  container: HTMLElement,
): { xOffset: number; colWidth: number }[] => {
  const elems = getHostElems(container);

  // Compute column widths
  const colWidths = elems.reduce(
    (acc, elem) => {
      const location = findLocation(elem);
      if (!location) return acc;
      const indexes = locationStringToIndexes(location);
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

  // Find the max colIndex to size the array
  const maxColIndex = Math.max(...Object.keys(colWidths).map(Number), 0);

  let xOffset = startX - gatePadding;
  const result: { xOffset: number; colWidth: number }[] = [];
  for (let colIndex = 0; colIndex <= maxColIndex; colIndex++) {
    const colWidth = colWidths[colIndex] ?? minGateWidth;
    result[colIndex] = { xOffset, colWidth };
    xOffset += colWidth + gatePadding * 2;
  }
  return result;
};

/**
 * Create a dropzone box element.
 *
 * @param colIndex The index of the column where the dropzone is located.
 * @param opIndex The index of the operation within the column.
 * @param colArray   An array of objects containing xOffset and colWidth for each column.
 * @param wireData The array of wire Y positions.
 * @param wireIndex The index of the wire for which the dropzone is created.
 * @param interColumn Whether the dropzone is between columns.
 *
 * @returns The created dropzone SVG element.
 */
const makeDropzoneBox = (
  colIndex: number,
  opIndex: number,
  colArray: { xOffset: number; colWidth: number }[],
  wireData: number[],
  wireIndex: number,
  interColumn: boolean,
): SVGElement => {
  const wireY = wireData[wireIndex];
  let xOffset: number, colWidth: number;

  if (colArray[colIndex]) {
    ({ xOffset, colWidth } = colArray[colIndex]);
  } else {
    // Compute offset for a hypothetical new last column
    const last = colArray[colArray.length - 1];
    if (last) {
      xOffset = last.xOffset + last.colWidth + gatePadding * 2;
    } else {
      // If there are no columns at all, start at initial offset
      xOffset = startX - gatePadding;
    }
    colWidth = minGateWidth;
  }

  const paddingY = 20;
  let dropzone = null;
  if (interColumn) {
    dropzone = box(
      xOffset - gatePadding * 2,
      wireY - paddingY,
      gatePadding * 4,
      paddingY * 2,
      "dropzone",
    );
  } else {
    dropzone = box(
      xOffset + gatePadding,
      wireY - paddingY,
      colWidth,
      paddingY * 2,
      "dropzone",
    );
  }
  dropzone.setAttribute("data-dropzone-location", `${colIndex},${opIndex}`);
  dropzone.setAttribute("data-dropzone-wire", `${wireIndex}`);
  dropzone.setAttribute("data-dropzone-inter-column", `${interColumn}`);
  return dropzone;
};

export {
  createDragzones,
  createGhostElement,
  createWireDropzone,
  removeAllWireDropzones,
  getColumnOffsetsAndWidths,
  makeDropzoneBox,
};
