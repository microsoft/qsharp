// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import cloneDeep from "lodash/cloneDeep";
import isEqual from "lodash/isEqual";
import { ComponentGrid, Operation, Qubit, Unitary } from "./circuit";
import { Sqore } from "./sqore";
import { toolboxGateDictionary } from "./panel";
import {
  getGateLocationString,
  findOperation,
  getToolboxElems,
  getGateElems,
  getHostElems,
  getWireData,
  locationStringToIndexes,
  findParentArray,
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
import { getMinMaxRegIdx } from "../../src/utils";

let events: CircuitEvents | null = null;

/**
 * Creates and attaches the events that allow editing of the circuit.
 *
 * @param container     HTML element for rendering visualization into
 * @param sqore         Sqore object
 */
const enableEvents = (
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
  private circuitSvg: SVGElement;
  private dropzoneLayer: SVGGElement;
  private wireData: number[];
  private selectedOperation: Operation | null = null;
  private selectedWire: number | null = null;
  private movingControl: boolean = false;
  private mouseUpOnCircuit: boolean = false;
  private dragging: boolean = false;
  private disableLeftAutoScroll: boolean = false;

  constructor(
    private container: HTMLElement,
    sqore: Sqore,
    useRefresh: () => void,
  ) {
    this.renderFn = useRefresh;

    this.circuitSvg = container.querySelector("svg[id]") as SVGElement;
    this.dropzoneLayer = container.querySelector(
      ".dropzone-layer",
    ) as SVGGElement;

    this.componentGrid = sqore.circuit.componentGrid;
    this.qubits = sqore.circuit.qubits;

    this.wireData = getWireData(this.container);

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
    this.disableLeftAutoScroll = false;
    this.movingControl = false;
    this.mouseUpOnCircuit = false;
  };

  /**
   * Enable auto-scrolling when dragging near the edges of the container
   */
  _enableAutoScroll() {
    const scrollSpeed = 10; // Pixels per frame
    const edgeThreshold = 50; // Distance from the edge to trigger scrolling

    // Utility function to find the nearest scrollable ancestor
    const getScrollableAncestor = (element: Element): HTMLElement => {
      let currentElement: Element | null = element;
      while (currentElement) {
        const overflowY = window.getComputedStyle(currentElement).overflowY;
        const overflowX = window.getComputedStyle(currentElement).overflowX;
        if (
          overflowY === "auto" ||
          overflowY === "scroll" ||
          overflowX === "auto" ||
          overflowX === "scroll"
        ) {
          return currentElement as HTMLElement;
        }
        currentElement = currentElement.parentElement;
      }
      return document.documentElement; // Fallback to the root element
    };

    const scrollableAncestor = getScrollableAncestor(this.circuitSvg);

    const onMouseMove = (ev: MouseEvent) => {
      const rect = scrollableAncestor.getBoundingClientRect();

      const topBoundary = rect.top;
      const bottomBoundary = rect.bottom;
      const leftBoundary = rect.left;
      const rightBoundary = rect.right;

      // If the mouse has moved past the left boundary, we want to re-enable left auto-scrolling
      if (
        this.disableLeftAutoScroll &&
        ev.clientX > leftBoundary + 3 * edgeThreshold
      ) {
        this.disableLeftAutoScroll = false;
      }

      // Check if the cursor is near the edges
      if (ev.clientY < topBoundary + edgeThreshold) {
        // Scroll up
        scrollableAncestor.scrollTop -= scrollSpeed;
      } else if (ev.clientY > bottomBoundary - edgeThreshold) {
        // Scroll down
        scrollableAncestor.scrollTop += scrollSpeed;
      }

      if (
        !this.disableLeftAutoScroll &&
        ev.clientX < leftBoundary + edgeThreshold
      ) {
        // Scroll left
        scrollableAncestor.scrollLeft -= scrollSpeed;
      } else if (ev.clientX > rightBoundary - edgeThreshold) {
        // Scroll right
        scrollableAncestor.scrollLeft += scrollSpeed;
      }
    };

    const onMouseUp = () => {
      // Remove the mousemove listener when dragging stops
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    // Add the mousemove listener when dragging starts
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }

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
        // Allow dragging even when initiated on the arg-button
        if ((ev.target as HTMLElement).classList.contains("arg-button")) {
          // Find the sibling element with the data-wire attribute
          const siblingWithWire = (
            ev.target as HTMLElement
          ).parentElement?.querySelector("[data-wire]");
          if (siblingWithWire) {
            const selectedWireStr = siblingWithWire.getAttribute("data-wire");
            this.selectedWire =
              selectedWireStr != null ? parseInt(selectedWireStr) : null;
          }
        }

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

        this._createGhostElement(ev);

        // Make sure the selectedOperation has location data
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

      // Enable arg-button behavior
      const argButtons = elem.querySelectorAll<SVGElement>(".arg-button");
      argButtons.forEach((argButton) => {
        argButton.classList.add("edit-mode");

        // Add click event to trigger promptForArguments
        argButton.addEventListener("click", async () => {
          if (this.selectedOperation == null) return;
          const params = this.selectedOperation.params;
          const displayArgs = argButton.textContent || "";
          if (params) {
            const args = await promptForArguments(params, [displayArgs]);
            if (args.length > 0) {
              this.selectedOperation.args = args;
              this.renderFn();
            }
          }
        });
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
    this.selectedOperation = toolboxGateDictionary[type];
    this.disableLeftAutoScroll = true;
    this._createGhostElement(ev);
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
            if (
              this.movingControl &&
              this.selectedOperation.kind === "unitary"
            ) {
              addControl(this.selectedOperation, targetWire);
              moveOperation(
                this,
                sourceLocation,
                targetLoc,
                this.selectedWire,
                targetWire,
                false,
                insertNewColumn,
              );
            } else {
              addOperation(
                this,
                this.selectedOperation,
                targetLoc,
                targetWire,
                insertNewColumn,
              );
            }
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
        // Determines if the operation is associated with the last qubit line
        const check = (op: Operation) => {
          const targets = op.kind === "measurement" ? op.results : op.targets;
          if (targets.some((reg) => reg.qubit == this.qubits.length - 1)) {
            return true;
          }
          const controls =
            op.kind === "measurement"
              ? op.qubits
              : op.kind === "ket"
                ? []
                : op.controls;
          if (
            controls &&
            controls.some((reg) => reg.qubit == this.qubits.length - 1)
          ) {
            return true;
          }
          return false;
        };

        // Count number of operations associated with the last qubit line
        const numOperations = this.componentGrid.reduce(
          (acc, column) =>
            acc + column.components.filter((op) => check(op)).length,
          0,
        );
        if (numOperations === 0) {
          this.qubits.pop();
          this.renderFn();
        } else {
          const message =
            numOperations === 1
              ? `There is ${numOperations} operation associated with the last qubit line. Do you want to remove it?`
              : `There are ${numOperations} operations associated with the last qubit line. Do you want to remove them?`;
          _createConfirmPrompt(message, (confirmed) => {
            if (confirmed) {
              // Remove all operations associated with the last qubit line
              findAndRemoveOperations(this.componentGrid, check);
              this.qubits.pop();
              this.renderFn();
            }
          });
        }
      });
      removeQubitLineButton.setAttribute("data-event-added", "true");
    }
  }

  /*****************
   *     Misc.     *
   *****************/

  /**
   * Creates a ghost element for visual feedback during dragging.
   * This function initializes the dragging state, enables auto-scrolling,
   * and creates a visual representation of the selected operation.
   *
   * @param ev - The mouse event that triggered the creation of the ghost element.
   */
  _createGhostElement(ev: MouseEvent) {
    if (this.selectedOperation == null) return;
    this.dragging = true;
    this._enableAutoScroll();
    createGhostElement(
      ev,
      this.container,
      this.selectedOperation,
      this.movingControl,
    );
  }

  /**
   * Start the process of adding a control to the selected operation.
   * This function creates dropzones for each wire that isn't already a target or control.
   *
   * @param selectedOperation - The unitary operation to which the control will be added.
   * @param selectedLocation - The location string of the selected operation.
   */
  _startAddingControl(selectedOperation: Unitary, selectedLocation: string) {
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
              const indexes = locationStringToIndexes(selectedLocation);
              const [columnIndex, position] = indexes[indexes.length - 1];
              const selectedOperationParent = findParentArray(
                this.componentGrid,
                selectedLocation,
              );
              if (!selectedOperationParent) return;

              const [minTarget, maxTarget] = getMinMaxRegIdx(
                selectedOperation,
                this.wireData.length,
              );
              selectedOperationParent[columnIndex].components.forEach(
                (op, opIndex) => {
                  if (opIndex === position) return; // Don't check the selected operation against itself
                  const [minOp, maxOp] = getMinMaxRegIdx(
                    op,
                    this.wireData.length,
                  );
                  // Check if selectedOperation's range overlaps with op's range
                  if (
                    (minOp >= minTarget && minOp <= maxTarget) ||
                    (maxOp >= minTarget && maxOp <= maxTarget) ||
                    (minTarget >= minOp && minTarget <= minOp) ||
                    (maxTarget >= maxOp && maxTarget <= maxOp)
                  ) {
                    // If they overlap, move the operation
                    selectedOperationParent[columnIndex].components.splice(
                      position,
                      1,
                    );
                    // Not sure if this check is needed as we already know there
                    // should be other operations in this column.
                    if (
                      selectedOperationParent[columnIndex].components.length ===
                      0
                    ) {
                      selectedOperationParent.splice(columnIndex, 1);
                    }

                    selectedOperationParent.splice(columnIndex, 0, {
                      components: [selectedOperation],
                    });
                  }
                },
              );

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

/**
 * Create a confirm dialog box
 * @param message - The message to display in the confirm dialog
 * @param callback - The callback function to handle the user's response (true for OK, false for Cancel)
 */
const _createConfirmPrompt = (
  message: string,
  callback: (confirmed: boolean) => void,
) => {
  // Create the confirm overlay
  const overlay = document.createElement("div");
  overlay.classList.add("prompt-overlay");
  overlay.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
  });

  // Create the confirm container
  const confirmContainer = document.createElement("div");
  confirmContainer.classList.add("prompt-container");

  // Create the message element
  const messageElem = document.createElement("div");
  messageElem.classList.add("prompt-message");
  messageElem.textContent = message;

  // Create the buttons container
  const buttonsContainer = document.createElement("div");
  buttonsContainer.classList.add("prompt-buttons");

  // Create the OK button
  const okButton = document.createElement("button");
  okButton.classList.add("prompt-button");
  okButton.textContent = "OK";
  okButton.addEventListener("click", () => {
    callback(true); // User confirmed
    document.body.removeChild(overlay);
  });

  // Create the Cancel button
  const cancelButton = document.createElement("button");
  cancelButton.classList.add("prompt-button");
  cancelButton.textContent = "Cancel";
  cancelButton.addEventListener("click", () => {
    callback(false); // User canceled
    document.body.removeChild(overlay);
  });

  buttonsContainer.appendChild(okButton);
  buttonsContainer.appendChild(cancelButton);

  confirmContainer.appendChild(messageElem);
  confirmContainer.appendChild(buttonsContainer);

  overlay.appendChild(confirmContainer);
  document.body.appendChild(overlay);

  // Remove focus from any currently focused element
  if (document.activeElement) {
    (document.activeElement as HTMLElement).blur();
  }
};

export { enableEvents, CircuitEvents };
