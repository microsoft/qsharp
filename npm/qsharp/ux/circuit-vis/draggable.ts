// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { ComponentGrid, Operation } from "./circuit";
import { gatePadding, minGateWidth, startX } from "./constants";
import { box, controlDot } from "./formatters/formatUtils";
import { formatGate } from "./formatters/gateFormatter";
import { toMetadata } from "./panel";
import { Sqore } from "./sqore";
import {
  findLocation,
  getHostElems,
  getMinMaxRegIdx,
  getToolboxElems,
  getWireData,
  locationStringToIndexes,
} from "./utils";

interface Context {
  container: HTMLElement;
  svg: SVGElement;
  operationGrid: ComponentGrid;
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
    container,
    svg,
    operationGrid: sqore.circuit.componentGrid,
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

  const { container, operationGrid, wireData, paddingY } = context;
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

  // let xOffset = regLineStart;
  let xOffset = startX - gatePadding;

  /**
   * Create a dropzone box element.
   *
   * @param wireIndex The index of the wire for which the dropzone is created.
   * @param colIndex The index of the column where the dropzone is located.
   * @param colWidth The width of the column.
   * @param opIndex The index of the operation within the column.
   * @param interColumn Whether the dropzone is between columns.
   * @returns The created dropzone SVG element.
   */
  const _makeDropzoneBox = (
    wireIndex: number,
    colIndex: number,
    colWidth: number,
    opIndex: number,
    interColumn: boolean,
  ): SVGElement => {
    const wireY = wireData[wireIndex];
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

  // Create dropzones for each intersection of columns and wires
  sortedColWidths.forEach(([colIndex, colWidth]) => {
    const columnOps = operationGrid[colIndex];
    let wireIndex = 0;

    columnOps.components.forEach((op, opIndex) => {
      const [minTarget, maxTarget] = getMinMaxRegIdx(op, wireData.length);
      // Add dropzones before the first target
      while (wireIndex <= maxTarget) {
        dropzoneLayer.appendChild(
          _makeDropzoneBox(wireIndex, colIndex, colWidth, opIndex, true),
        );
        // We don't want to make a central zone if the spot is occupied by a gate or its connecting lines
        if (wireIndex < minTarget) {
          dropzoneLayer.appendChild(
            _makeDropzoneBox(wireIndex, colIndex, colWidth, opIndex, false),
          );
        }

        wireIndex++;
      }
    });

    // Add dropzones after the last target
    while (wireIndex < wireData.length) {
      dropzoneLayer.appendChild(
        _makeDropzoneBox(
          wireIndex,
          colIndex,
          colWidth,
          columnOps.components.length,
          true,
        ),
      );
      dropzoneLayer.appendChild(
        _makeDropzoneBox(
          wireIndex,
          colIndex,
          colWidth,
          columnOps.components.length,
          false,
        ),
      );

      wireIndex++;
    }
    xOffset += colWidth + gatePadding * 2;
  });

  // This assumes column indexes are continuous
  const endColIndex = sortedColWidths.length;

  // Add remaining dropzones to allow users to add gates to the end of the circuit
  for (let wireIndex = 0; wireIndex < wireData.length; wireIndex++) {
    const dropzone = _makeDropzoneBox(wireIndex, endColIndex, 0, 0, true);
    // Note: the last column should have the shape of an inter-column dropzone, but
    // we don't want to attach the inter-column logic to it.
    dropzone.setAttribute("data-dropzone-inter-column", "false");
    dropzoneLayer.appendChild(dropzone);
  }

  return dropzoneLayer;
};

export {
  extensionDraggable,
  createGhostElement,
  createWireDropzone,
  removeAllWireDropzones,
};
