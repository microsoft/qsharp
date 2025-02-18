// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import cloneDeep from "lodash/cloneDeep";
import isEqual from "lodash/isEqual";
import { Operation, Qubit } from "./circuit";
import { Sqore } from "./sqore";
import { defaultGateDictionary } from "./panel";
import {
  getGateTargets,
  getGateLocationString,
  findParentOperation,
  findOperation,
  getToolboxElems,
  getGateElems,
  getHostElems,
  getWireData,
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
import {
  createGhostElement,
  createWireDropzone,
  removeAllWireDropzones,
} from "./draggable";

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
  operations: Operation[][];
  qubits: Qubit[];
  private container: HTMLElement;
  private circuitSvg: SVGElement;
  private dropzoneLayer: SVGGElement;
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
    this.wireData = getWireData(this.container);
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
    removeAllWireDropzones(this.circuitSvg);
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
    const elems = getHostElems(this.container);
    elems.forEach((elem) => {
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
    const elems = getGateElems(this.container);
    elems.forEach((elem) => {
      elem?.addEventListener("mousedown", (ev: MouseEvent) => {
        if (ev.button !== 0) return;
        ev.stopPropagation();
        removeAllWireDropzones(this.circuitSvg);
        if (elem.getAttribute("data-expanded") !== "true") {
          const selectedLocation = elem.getAttribute("data-location");
          this.selectedOperation = findOperation(
            this.operations,
            selectedLocation,
          );
          if (this.selectedOperation == null || !selectedLocation) return;

          createGhostElement(
            ev,
            this.container,
            this.selectedOperation,
            this.movingControl,
          );

          // ToDo: This shouldn't be necessary. Find out why all the operations are missing their dataAttributes from sqore
          if (this.selectedOperation.dataAttributes == null) {
            this.selectedOperation.dataAttributes = {
              location: selectedLocation,
            };
          } else {
            this.selectedOperation.dataAttributes["location"] =
              selectedLocation;
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
    createGhostElement(ev, this.container, this.selectedOperation, false);
  };

  /**
   * Add events for gates in the toolbox
   */
  _addToolboxElementsEvents() {
    const elems = getToolboxElems(this.container);
    elems.forEach((elem) => {
      elem.addEventListener("mousedown", this.toolboxMousedownHandler);
    });
  }

  /**
   * Remove events for gates in the toolbox
   */
  _removeToolboxElementsEvents() {
    const elems = getToolboxElems(this.container);
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
              const parentOperation = findParentOperation(
                this.operations,
                sourceLocation,
              );
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
        const dropzone = createWireDropzone(
          this.circuitSvg,
          this.wireData,
          wireIndex,
        );
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
      const dropzone = createWireDropzone(
        this.circuitSvg,
        this.wireData,
        control.qId,
      );
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
}

export { extensionEvents, CircuitEvents };
