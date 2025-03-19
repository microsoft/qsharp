// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import cloneDeep from "lodash/cloneDeep";
import isEqual from "lodash/isEqual";
import { ComponentGrid, Operation, Qubit, Unitary } from "./circuit";
import { Sqore } from "./sqore";
import { defaultGateDictionary } from "./panel";
import {
  getGateLocationString,
  findOperation,
  getToolboxElems,
  getGateElems,
  getHostElems,
  getWireData,
} from "./utils";
import { addContextMenuToHostElem, promptForArguments } from "./contextMenu";
import {
  addControl,
  addOperation,
  findAndRemoveOperations,
  moveOperation,
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
  componentGrid: ComponentGrid;
  qubits: Qubit[];
  private container: HTMLElement;
  private circuitSvg: SVGElement;
  private dropzoneLayer: SVGGElement;
  private wireData: number[];
  private selectedOperation: Operation | null;
  private selectedWire: number | null;
  private movingControl: boolean;
  private mouseUpOnCircuit: boolean;
  private dragging: boolean;

  constructor(container: HTMLElement, sqore: Sqore, useRefresh: () => void) {
    this.renderFn = useRefresh;

    this.container = container;
    this.circuitSvg = container.querySelector("svg[id]") as SVGElement;
    this.dropzoneLayer = container.querySelector(
      ".dropzone-layer",
    ) as SVGGElement;

    this.componentGrid = sqore.circuit.componentGrid;
    this.qubits = sqore.circuit.qubits;

    this.wireData = getWireData(this.container);
    this.selectedOperation = null;
    this.selectedWire = null;

    this.movingControl = false;
    this.mouseUpOnCircuit = false;
    this.dragging = false;

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

  documentMouseupHandler = (ev: MouseEvent) => {
    const copying = ev.ctrlKey;
    this.container.classList.remove("moving", "copying");
    if (this.container) {
      const ghostElem = this.container.querySelector(".ghost");
      if (ghostElem) {
        this.container.removeChild(ghostElem);
      }

      // Handle deleting operations that have been dragged outside the circuit
      if (!this.mouseUpOnCircuit && this.dragging && !copying) {
        const selectedLocation = this.selectedOperation
          ? getGateLocationString(this.selectedOperation)
          : null;
        if (this.selectedOperation != null && selectedLocation != null) {
          // We are dragging a gate with a location (not from toolbox) outside the circuit
          // If we are moving a control, remove it from the selectedOperation
          if (
            this.movingControl &&
            this.selectedOperation.kind === "unitary" &&
            this.selectedOperation.controls != null &&
            this.selectedWire != null
          ) {
            const controlIndex = this.selectedOperation.controls.findIndex(
              (control) => control.qubit === this.selectedWire,
            );
            if (controlIndex !== -1)
              this.selectedOperation.controls.splice(controlIndex, 1);
          } else {
            // Otherwise, remove the selectedOperation
            removeOperation(this, selectedLocation);
          }
          this.renderFn();
        }
      }
    }
    this.dragging = false;
    this.movingControl = false;
    this.mouseUpOnCircuit = false;
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

    this.circuitSvg.addEventListener("mouseup", () => {
      this.mouseUpOnCircuit = true;
    });
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

      addContextMenuToHostElem(this, elem);
    });
  }

  /**
   * Add events for circuit objects in the circuit
   */
  _addGateElementsEvents() {
    const elems = getGateElems(this.container);
    elems.forEach((elem) => {
      elem?.addEventListener("mousedown", (ev: MouseEvent) => {
        let selectedLocation = null;
        if (elem.getAttribute("data-expanded") !== "true") {
          selectedLocation = elem.getAttribute("data-location");
          this.selectedOperation = findOperation(
            this.componentGrid,
            selectedLocation,
          );
        }
        if (ev.button !== 0) return;
        ev.stopPropagation();
        removeAllWireDropzones(this.circuitSvg);
        if (this.selectedOperation == null || !selectedLocation) return;

        this.dragging = true;
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
          this.selectedOperation.dataAttributes["location"] = selectedLocation;
        }

        this.container.classList.add("moving");
        this.dropzoneLayer.style.display = "block";
      });
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
    this.dragging = true;
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
      dropzoneElem.addEventListener("mouseup", async (ev: MouseEvent) => {
        const copying = ev.ctrlKey;
        const originalGrid = cloneDeep(this.componentGrid);
        const targetLoc = dropzoneElem.getAttribute("data-dropzone-location");
        const insertNewColumn =
          dropzoneElem.getAttribute("data-dropzone-inter-column") == "true" ||
          false;
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
          if (
            this.selectedOperation.params != undefined &&
            (this.selectedOperation.args === undefined ||
              this.selectedOperation.args.length === 0)
          ) {
            // Prompt for arguments and wait for user input
            const args = await promptForArguments(
              this.selectedOperation.params,
            );
            if (!args || args.length === 0) {
              // User canceled the prompt, exit early
              return;
            }

            // Create a deep copy of the source operation
            this.selectedOperation = JSON.parse(
              JSON.stringify(this.selectedOperation),
            );
            if (this.selectedOperation == null) return;

            // Assign the arguments to the selected operation
            this.selectedOperation.args = args;
          }

          // Add a new operation from the toolbox
          addOperation(
            this,
            this.selectedOperation,
            targetLoc,
            targetWire,
            insertNewColumn,
          );
        } else if (sourceLocation && this.selectedWire != null) {
          if (copying) {
            addOperation(
              this,
              this.selectedOperation,
              targetLoc,
              targetWire,
              insertNewColumn,
            );
          } else {
            moveOperation(
              this,
              sourceLocation,
              targetLoc,
              this.selectedWire,
              targetWire,
              this.movingControl,
              insertNewColumn,
            );
          }
        }

        this.selectedWire = null;
        this.selectedOperation = null;
        this.movingControl = false;

        if (isEqual(originalGrid, this.componentGrid) === false)
          this.renderFn();
      });
    });
  }

  /**
   * Add event listeners for the buttons to add or remove qubit lines.
   * The add button will append a new qubit line to the circuit.
   * The remove button will remove the last qubit line from the circuit,
   * along with any operations associated with it.
   */
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
          const targets = op.kind === "measurement" ? op.results : op.targets;
          if (targets.some((reg) => reg.qubit == this.qubits.length - 1)) {
            return true;
          }
          const controls = op.kind === "measurement" ? op.qubits : op.controls;
          if (
            controls &&
            controls.some((reg) => reg.qubit == this.qubits.length - 1)
          ) {
            return true;
          }
          return false;
        };
        findAndRemoveOperations(this.componentGrid, check);
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
   * @param selectedOperation - The unitary operation to which the control will be added.
   */
  _startAddingControl(selectedOperation: Unitary) {
    this.selectedOperation = selectedOperation;
    this.container.classList.add("adding-control");

    // Create dropzones for each wire that isn't already a target or control
    for (let wireIndex = 0; wireIndex < this.wireData.length; wireIndex++) {
      const isTarget = this.selectedOperation?.targets.some(
        (target) => target.qubit === wireIndex,
      );
      const isControl = this.selectedOperation?.controls?.some(
        (control) => control.qubit === wireIndex,
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
          if (
            this.selectedOperation != null &&
            this.selectedOperation.kind === "unitary"
          ) {
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
   * @param selectedOperation - The unitary operation from which the control will be removed.
   */
  _startRemovingControl(selectedOperation: Unitary) {
    this.selectedOperation = selectedOperation;
    this.container.classList.add("removing-control");

    // Create dropzones only for wires that the selectedOperation has a control
    this.selectedOperation.controls?.forEach((control) => {
      const dropzone = createWireDropzone(
        this.circuitSvg,
        this.wireData,
        control.qubit,
      );
      dropzone.addEventListener("mousedown", (ev: MouseEvent) =>
        ev.stopPropagation(),
      );
      dropzone.addEventListener("click", () => {
        if (
          this.selectedOperation != null &&
          this.selectedOperation.kind === "unitary"
        ) {
          const successful = removeControl(
            this.selectedOperation,
            control.qubit,
          );
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
