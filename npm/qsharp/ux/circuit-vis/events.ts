// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import cloneDeep from "lodash/cloneDeep";
import isEqual from "lodash/isEqual";
import { Operation, Qubit } from "./circuit";
import { Sqore } from "./sqore";
import { _formatGate } from "./formatters/gateFormatter";
import { box, controlDot } from "./formatters/formatUtils";
import { defaultGateDictionary, toMetadata } from "./panel";
import {
  locationStringToIndexes,
  getGateTargets,
  getGateLocationString,
} from "./utils";
import { addContextMenuToGateElem } from "./contextMenu";
import {
  addControl,
  addOperation,
  findAndRemoveOperations,
  moveX,
  removeControl,
  removeOperation,
} from "./circuitManipulation";

let events: CircuitEvents | null = null;

const extensionEvents = (
  container: HTMLElement,
  sqore: Sqore,
  useRefresh: () => void,
): void => {
  if (events != null) {
    events.dispose();
  }
  events = new CircuitEvents(container, sqore, useRefresh);
};

class CircuitEvents {
  renderFn: () => void;
  private container: HTMLElement;
  private circuitSvg: SVGElement;
  private dropzoneLayer: SVGGElement;
  operations: Operation[];
  qubits: Qubit[];
  private wireData: number[];
  private selectedOperation: Operation | null;
  private selectedWire: number | null;
  private movingControl: boolean;

  constructor(container: HTMLElement, sqore: Sqore, useRefresh: () => void) {
    this.renderFn = useRefresh;
    this.container = container;
    this.circuitSvg = container.querySelector("svg[id]") as SVGElement;
    this.dropzoneLayer = container.querySelector(
      ".dropzone-layer",
    ) as SVGGElement;
    this.operations = sqore.circuit.operations;
    this.qubits = sqore.circuit.qubits;
    this.wireData = this._wireData();
    this.selectedOperation = null;
    this.selectedWire = null;
    this.movingControl = false;

    this._addContextMenuEvent();
    this._addDropzoneLayerEvents();
    this._addHostElementsEvents();
    this._addGateElementsEvents();
    this._addToolboxElementsEvents();
    this._addDropzoneElementsEvents();
    this._addQubitLineControlEvents();
    this._addDocumentEvents();
  }

  /**
   * Dispose the CircuitEvents instance and remove event listeners
   */
  dispose() {
    this._removeToolboxElementsEvents();
    this._removeDocumentEvents();
  }

  /***************************
   * Events Adding Functions *
   ***************************/

  documentKeydownHandler = (ev: KeyboardEvent) => {
    const selectedLocation = this.selectedOperation
      ? getGateLocationString(this.selectedOperation)
      : null;
    if (ev.ctrlKey && selectedLocation) {
      this.container.classList.remove("moving");
      this.container.classList.add("copying");
    } else if (ev.key == "Delete" && selectedLocation) {
      removeOperation(this, selectedLocation);
      this.renderFn();
    }
  };

  documentKeyupHandler = (ev: KeyboardEvent) => {
    const selectedLocation = this.selectedOperation
      ? getGateLocationString(this.selectedOperation)
      : null;
    if (ev.ctrlKey && selectedLocation) {
      this.container.classList.remove("copying");
      this.container.classList.add("moving");
    }
  };

  documentMousedownHandler = () => {
    this._removeAllWireDropzones();
  };

  documentMouseupHandler = () => {
    this.container.classList.remove("moving", "copying");
    this.movingControl = false;
    if (this.container) {
      const ghostElem = this.container.querySelector(".ghost");
      if (ghostElem) {
        this.container.removeChild(ghostElem);
      }
    }
  };

  /**
   * Add events for document
   */
  _addDocumentEvents() {
    document.addEventListener("keydown", this.documentKeydownHandler);
    document.addEventListener("keyup", this.documentKeyupHandler);
    document.addEventListener("mouseup", this.documentMouseupHandler);
    document.addEventListener("mousedown", this.documentMousedownHandler);
  }

  /**
   * Remove events for document
   */
  _removeDocumentEvents() {
    document.removeEventListener("keydown", this.documentKeydownHandler);
    document.removeEventListener("keyup", this.documentKeyupHandler);
    document.removeEventListener("mouseup", this.documentMouseupHandler);
    document.removeEventListener("mousedown", this.documentMousedownHandler);
  }

  /**
   * Add events specifically for dropzoneLayer
   */
  _addDropzoneLayerEvents() {
    this.container.addEventListener(
      "mouseup",
      () => (this.dropzoneLayer.style.display = "none"),
    );
  }

  /**
   * Disable contextmenu default behaviors
   */
  _addContextMenuEvent() {
    this.container.addEventListener("contextmenu", (ev: MouseEvent) => {
      ev.preventDefault();
    });
  }

  /**
   * Add events for circuit objects in the circuit
   */
  _addHostElementsEvents() {
    const elems = this._hostElems();
    elems.forEach((elem) => {
      // DEBUG:
      // elem.addEventListener("mouseover", () => {
      //   const gate = this._findGateElem(elem);
      //   if (gate == null) return;
      //   const gateLoc = gate.getAttribute("data-location");
      //   const gateWireStr = elem.getAttribute("data-wire");
      //   console.log("Location: ", gateLoc, " Wire: ", gateWireStr);
      // });

      elem.addEventListener("mousedown", (ev: MouseEvent) => {
        if (ev.button !== 0) return;
        if (elem.classList.contains("control-dot")) {
          this.movingControl = true;
        }
        const selectedWireStr = elem.getAttribute("data-wire");
        this.selectedWire =
          selectedWireStr != null ? parseInt(selectedWireStr) : null;
      });
    });
  }

  /**
   * Add events for circuit objects in the circuit
   */
  _addGateElementsEvents() {
    const elems = this._gateElems();
    elems.forEach((elem) => {
      elem?.addEventListener("mousedown", (ev: MouseEvent) => {
        if (ev.button !== 0) return;
        ev.stopPropagation();
        this._removeAllWireDropzones();
        if (elem.getAttribute("data-expanded") !== "true") {
          const selectedLocation = elem.getAttribute("data-location");
          this.selectedOperation = this._findOperation(selectedLocation);

          this.createGhostElement(ev);

          // ToDo: This shouldn't be necessary. Find out why all the operations are missing their dataAttributes from sqore
          if (this.selectedOperation && selectedLocation) {
            if (this.selectedOperation.dataAttributes == null) {
              this.selectedOperation.dataAttributes = {
                location: selectedLocation,
              };
            } else {
              this.selectedOperation.dataAttributes["location"] =
                selectedLocation;
            }
          }
          this.container.classList.add("moving");
          this.dropzoneLayer.style.display = "block";
        }
      });

      addContextMenuToGateElem(this, elem);
    });
  }

  toolboxMousedownHandler = (ev: MouseEvent) => {
    if (ev.button !== 0) return;
    this.container.classList.add("moving");
    this.dropzoneLayer.style.display = "block";
    const elem = ev.currentTarget as HTMLElement;
    const type = elem.getAttribute("data-type");
    if (type == null) return;
    this.selectedOperation = defaultGateDictionary[type];
    this.createGhostElement(ev);
  };

  /**
   * Add events for gates in the toolbox
   */
  _addToolboxElementsEvents() {
    const elems = this._toolboxElems();
    elems.forEach((elem) => {
      elem.addEventListener("mousedown", this.toolboxMousedownHandler);
    });
  }

  /**
   * Remove events for gates in the toolbox
   */
  _removeToolboxElementsEvents() {
    const elems = this._toolboxElems();
    elems.forEach((elem) => {
      elem.removeEventListener("mousedown", this.toolboxMousedownHandler);
    });
  }

  /**
   * Add events for dropzone elements
   */
  _addDropzoneElementsEvents() {
    const dropzoneElems =
      this.dropzoneLayer.querySelectorAll<SVGRectElement>(".dropzone");
    dropzoneElems.forEach((dropzoneElem) => {
      // DEBUG:
      // dropzoneElem.addEventListener("mouseover", () => {
      //   const targetLoc = dropzoneElem.getAttribute("data-dropzone-location");
      //   const targetWireStr = dropzoneElem.getAttribute("data-dropzone-wire");
      //   console.log("Location: ", targetLoc, " Wire: ", targetWireStr);
      // });

      dropzoneElem.addEventListener("mouseup", (ev: MouseEvent) => {
        const originalOperations = cloneDeep(this.operations);
        const targetLoc = dropzoneElem.getAttribute("data-dropzone-location");
        const targetWireStr = dropzoneElem.getAttribute("data-dropzone-wire");
        const targetWire =
          targetWireStr != null ? parseInt(targetWireStr) : null;

        if (
          targetLoc == null ||
          targetWire == null ||
          this.selectedOperation == null
        )
          return;
        const sourceLocation = getGateLocationString(this.selectedOperation);

        if (sourceLocation == null) {
          // Add a new operation from the toolbox
          addOperation(this, this.selectedOperation, targetLoc, targetWire);
          // const newOperation = this._addOperation(this.selectedOperation, targetLoc, targetWire);
          // if (newOperation) {
          //     this._moveY(targetWire, newOperation, this.wireData.length);
          // }
        } else if (sourceLocation && this.selectedWire != null) {
          if (ev.ctrlKey) {
            addOperation(this, this.selectedOperation, targetLoc, targetWire);
          } else {
            const newOperation = moveX(
              this,
              sourceLocation,
              targetLoc,
              targetWire,
            );
            if (newOperation) {
              if (!newOperation.isMeasurement) {
                if (this.movingControl) {
                  newOperation.controls?.forEach((control) => {
                    if (control.qId === this.selectedWire) {
                      control.qId = targetWire;
                    }
                  });
                  newOperation.controls = newOperation.controls?.sort(
                    (a, b) => a.qId - b.qId,
                  );
                } else {
                  newOperation.targets = [{ qId: targetWire, type: 0 }];
                }
              }
              //this._moveY(targetWire - this.selectedWire, newOperation, this.wireData.length);
              const parentOperation = this._findParentOperation(sourceLocation);
              if (parentOperation) {
                parentOperation.targets = getGateTargets(parentOperation);
              }
            }
          }
        }

        this.selectedWire = null;
        this.selectedOperation = null;
        this.movingControl = false;

        if (isEqual(originalOperations, this.operations) === false)
          this.renderFn();
      });
    });
  }

  _addQubitLineControlEvents() {
    const addQubitLineButton = this.container.querySelector(".add-qubit-line");
    const removeQubitLineButton =
      this.container.querySelector(".remove-qubit-line");

    if (
      addQubitLineButton &&
      !addQubitLineButton.hasAttribute("data-event-added")
    ) {
      addQubitLineButton.addEventListener("click", () => {
        this.qubits.push({ id: this.qubits.length });
        this.renderFn();
      });
      addQubitLineButton.setAttribute("data-event-added", "true");
    }

    if (
      removeQubitLineButton &&
      !removeQubitLineButton.hasAttribute("data-event-added")
    ) {
      removeQubitLineButton.addEventListener("click", () => {
        const check = (op: Operation) => {
          if (op.targets.some((reg) => reg.qId == this.qubits.length - 1)) {
            return true;
          }
          if (
            op.controls &&
            op.controls.some((reg) => reg.qId == this.qubits.length - 1)
          ) {
            return true;
          }
          return false;
        };
        findAndRemoveOperations(this.operations, check);
        this.qubits.pop();
        this.renderFn();
      });
      removeQubitLineButton.setAttribute("data-event-added", "true");
    }
  }

  /**********************
   *  Finder Functions  *
   **********************/

  /**
   * Find the surrounding gate element of host element
   */
  _findGateElem(hostElem: SVGElement): SVGElement | null {
    return hostElem.closest<SVGElement>("[data-location]");
  }

  /**
   * Find location of the gate surrounding a host element
   */
  _findLocation(hostElem: SVGElement) {
    const gateElem = this._findGateElem(hostElem);
    return gateElem != null ? gateElem.getAttribute("data-location") : null;
  }

  /**
   * Find the parent operation of the operation specified by location
   */
  _findParentOperation(location: string | null): Operation | null {
    if (!location) return null;

    const indexes = locationStringToIndexes(location);
    indexes.pop();
    const lastIndex = indexes.pop();

    if (lastIndex == null) return null;

    let parentOperation = this.operations;
    for (const index of indexes) {
      parentOperation = parentOperation[index].children || parentOperation;
    }
    return parentOperation[lastIndex];
  }

  /**
   * Find the parent array of an operation based on its location
   */
  _findParentArray(location: string | null): Operation[] | null {
    if (!location) return null;

    const indexes = locationStringToIndexes(location);
    indexes.pop(); // The last index refers to the operation itself, remove it so that the last index instead refers to the parent operation

    let parentArray = this.operations;
    for (const index of indexes) {
      parentArray = parentArray[index].children || parentArray;
    }
    return parentArray;
  }

  /**
   * Find an operation based on its location
   */
  _findOperation(location: string | null): Operation | null {
    if (!location) return null;

    const index = locationStringToIndexes(location).pop();
    const operationParent = this._findParentArray(location);

    if (operationParent == null || index == null) return null;

    return operationParent[index];
  }

  /*****************
   *     Misc.     *
   *****************/

  /**
   * Start the process of adding a control to the selected operation.
   * This function creates dropzones for each wire that isn't already a target or control.
   *
   * @param selectedOperation - The operation to which the control will be added.
   */
  _startAddingControl(selectedOperation: Operation) {
    this.selectedOperation = selectedOperation;
    this.container.classList.add("adding-control");

    // Create dropzones for each wire that isn't already a target or control
    for (let wireIndex = 0; wireIndex < this.wireData.length; wireIndex++) {
      const isTarget = this.selectedOperation?.targets.some(
        (target) => target.qId === wireIndex,
      );
      const isControl = this.selectedOperation?.controls?.some(
        (control) => control.qId === wireIndex,
      );

      if (!isTarget && !isControl) {
        const dropzone = this._createWireDropzone(wireIndex);
        dropzone.addEventListener("mousedown", (ev: MouseEvent) =>
          ev.stopPropagation(),
        );
        dropzone.addEventListener("click", () => {
          if (this.selectedOperation != null) {
            const successful = addControl(this.selectedOperation, wireIndex);
            this.selectedOperation = null;
            this.container.classList.remove("adding-control");
            if (successful) {
              this.renderFn();
            }
          }
        });
        this.circuitSvg.appendChild(dropzone);
      }
    }
  }

  /**
   * Start the process of removing a control from the selected operation.
   * This function creates dropzones only for wires that the selected operation has a control.
   *
   * @param selectedOperation - The operation from which the control will be removed.
   */
  _startRemovingControl(selectedOperation: Operation) {
    this.selectedOperation = selectedOperation;
    this.container.classList.add("removing-control");

    // Create dropzones only for wires that the selectedOperation has a control
    this.selectedOperation.controls?.forEach((control) => {
      const dropzone = this._createWireDropzone(control.qId);
      dropzone.addEventListener("mousedown", (ev: MouseEvent) =>
        ev.stopPropagation(),
      );
      dropzone.addEventListener("click", () => {
        if (this.selectedOperation != null) {
          const successful = removeControl(this.selectedOperation, control.qId);
          this.selectedOperation = null;
          this.container.classList.remove("removing-control");
          if (successful) {
            this.renderFn();
          }
        }
      });
      this.circuitSvg.appendChild(dropzone);
    });
  }

  createGhostElement(ev: MouseEvent) {
    const ghost = this.movingControl
      ? controlDot(0, 0)
      : (() => {
          const ghostMetadata = toMetadata(this.selectedOperation!, 0, 0);
          return _formatGate(ghostMetadata).cloneNode(true) as SVGElement;
        })();

    // Generate svg element to wrap around ghost element
    const svgElem = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "svg",
    );
    svgElem.append(ghost);

    // Generate div element to wrap around svg element
    const divElem = document.createElement("div");
    divElem.classList.add("ghost");
    divElem.appendChild(svgElem);

    if (this.container) {
      this.container.appendChild(divElem);

      // Now that the element is appended to the DOM, get its dimensions
      const ghostRect = ghost.getBoundingClientRect();
      const ghostWidth = ghostRect.width;
      const ghostHeight = ghostRect.height;

      const updateDivLeftTop = (ev: MouseEvent) => {
        divElem.style.left = `${ev.clientX + window.scrollX - ghostWidth / 2}px`;
        divElem.style.top = `${ev.clientY + window.scrollY - ghostHeight / 2}px`;
      };

      updateDivLeftTop(ev);

      this.container.addEventListener("mousemove", updateDivLeftTop);
    } else {
      console.error("container not found");
    }
  }

  /**
   * Create a dropzone element that spans the length of the wire
   */
  _createWireDropzone(wireIndex: number): SVGElement {
    const wireY = this.wireData[wireIndex];
    const svgWidth = Number(this.circuitSvg.getAttribute("width"));
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
  }

  /**
   * Remove all wire dropzones
   */
  _removeAllWireDropzones() {
    const dropzones = this.circuitSvg.querySelectorAll(".dropzone-full-wire");
    dropzones.forEach((elem) => {
      elem.parentNode?.removeChild(elem);
    });
  }

  /**
   * Get list of y values based on circuit wires
   */
  _wireData(): number[] {
    // elems include qubit wires and lines of measure gates
    const elems = this.container.querySelectorAll<SVGGElement>(
      "svg[id] > g:nth-child(3) > g",
    );
    // filter out <g> elements having more than 2 elements because
    // qubit wires contain only 2 elements: <line> and <text>
    // lines of measure gates contain 4 <line> elements
    const wireElems = Array.from(elems).filter(
      (elem) => elem.childElementCount < 3,
    );
    const wireData = wireElems.map((wireElem) => {
      const lineElem = wireElem.children[0] as SVGLineElement;
      return Number(lineElem.getAttribute("y1"));
    });
    return wireData;
  }

  /**
   * Get list of toolbox items
   */
  _toolboxElems(): SVGGraphicsElement[] {
    return Array.from(
      this.container.querySelectorAll<SVGGraphicsElement>("[toolbox-item]"),
    );
  }

  /**
   * Get list of host elements that dropzones can be attached to
   */
  _hostElems(): SVGGraphicsElement[] {
    return Array.from(
      this.circuitSvg.querySelectorAll<SVGGraphicsElement>(
        '[class^="gate-"]:not(.gate-control, .gate-swap), .control-dot, .oplus, .cross',
      ),
    );
  }

  /**
   * Get list of gate elements from the circuit, but not the toolbox
   */
  _gateElems(): SVGGraphicsElement[] {
    return Array.from(
      this.circuitSvg.querySelectorAll<SVGGraphicsElement>(".gate"),
    );
  }
}

export { extensionEvents, CircuitEvents };
