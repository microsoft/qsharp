// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import cloneDeep from "lodash/cloneDeep";
import { Operation, Qubit } from "./circuit";
import { Register } from "./register";
import { Sqore } from "./sqore";
import isEqual from "lodash/isEqual";
import { _formatGate } from "./formatters/gateFormatter";
import { box, controlDot } from "./formatters/formatUtils";
import { defaultGateDictionary, toMetadata } from "./panel";

const extensionEvents = (
  container: HTMLElement,
  sqore: Sqore,
  useRefresh: () => void,
): void => {
  const events = new CircuitEvents(container, sqore, useRefresh);

  events._addContextMenuEvent();
  events._addDropzoneLayerEvents();
  events._addHostElementsEvents();
  events._addGateElementsEvents();
  events._addToolboxElementsEvents();
  events._addDropzoneElementsEvents();
  events._addQubitLineControlEvents();
  events._addDocumentEvents();
};

class CircuitEvents {
  private container: HTMLElement;
  private circuitSvg: SVGElement;
  private dropzoneLayer: SVGGElement;
  private operations: Operation[];
  private qubits: Qubit[];
  private wireData: number[];
  private renderFn: () => void;
  private selectedOperation: Operation | null;
  private selectedWire: number | null;
  private movingControl: boolean;

  constructor(container: HTMLElement, sqore: Sqore, useRefresh: () => void) {
    this.container = container;
    this.circuitSvg = container.querySelector("svg[id]") as SVGElement;
    this.dropzoneLayer = container.querySelector(
      ".dropzone-layer",
    ) as SVGGElement;
    this.operations = sqore.circuit.operations;
    this.qubits = sqore.circuit.qubits;
    this.wireData = this._wireData();
    this.renderFn = useRefresh;
    this.selectedOperation = null;
    this.selectedWire = null;
    this.movingControl = false;
  }

  /**
   * Generate an array of y values based on circuit wires
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

  /***************************
   * Events Adding Functions *
   ***************************/

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
   * Add events for document
   */
  _addDocumentEvents() {
    document.addEventListener("keydown", (ev: KeyboardEvent) => {
      const selectedLocation = this.selectedOperation
        ? this._getLocation(this.selectedOperation)
        : null;
      if (ev.ctrlKey && selectedLocation) {
        this.container.classList.remove("moving");
        this.container.classList.add("copying");
      } else if (ev.key == "Delete" && selectedLocation) {
        this._removeOperation(selectedLocation);
        this.renderFn();
      }
    });

    document.addEventListener("keyup", (ev: KeyboardEvent) => {
      const selectedLocation = this.selectedOperation
        ? this._getLocation(this.selectedOperation)
        : null;
      if (ev.ctrlKey && selectedLocation) {
        this.container.classList.remove("copying");
        this.container.classList.add("moving");
      }
    });

    document.addEventListener("mouseup", () => {
      this.container.classList.remove("moving", "copying");
      this.movingControl = false;
      if (this.container) {
        const ghostElem = this.container.querySelector(".ghost");
        if (ghostElem) {
          this.container.removeChild(ghostElem);
        }
      }
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
    const elems = this._hostElems();
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
    const elems = this._gateElems();
    elems.forEach((elem) => {
      elem?.addEventListener("mousedown", (ev: MouseEvent) => {
        if (ev.button !== 0) return;
        ev.stopPropagation();
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

      this._addContextMenuToGateElem(elem);
    });
  }

  _addContextMenuToGateElem(elem: SVGGraphicsElement) {
    elem?.addEventListener("contextmenu", (ev: MouseEvent) => {
      ev.preventDefault();

      // Remove any existing context menu
      const existingContextMenu = document.querySelector(".context-menu");
      if (existingContextMenu) {
        document.body.removeChild(existingContextMenu);
      }

      const selectedLocation = elem.getAttribute("data-location");
      const selectedOperation = this._findOperation(selectedLocation);
      if (!selectedOperation || !selectedLocation) return;

      const contextMenu = document.createElement("div");
      contextMenu.classList.add("context-menu");
      contextMenu.style.top = `${ev.clientY}px`;
      contextMenu.style.left = `${ev.clientX}px`;

      const adjointOption = this.createContextMenuItem("Toggle Adjoint", () => {
        selectedOperation.isAdjoint = !selectedOperation.isAdjoint;
        this.renderFn();
      });

      const addControlOption = this.createContextMenuItem("Add control", () => {
        this._startAddingControl(selectedOperation);
      });

      let removeControlOption: HTMLDivElement | null = null;
      if (selectedOperation.controls && selectedOperation.controls.length > 0) {
        removeControlOption = this.createContextMenuItem(
          "Remove control",
          () => {
            this._startRemovingControl(selectedOperation);
          },
        );
      }

      const promptArgOption = this.createContextMenuItem(
        "Edit Argument",
        () => {
          this._createCustomPrompt(
            "Argument for Gate:",
            (userInput) => {
              if (userInput !== null) {
                if (userInput == "") {
                  selectedOperation.displayArgs = undefined;
                } else {
                  selectedOperation.displayArgs = userInput;
                }
              }
              this.renderFn();
            },
            selectedOperation.displayArgs,
          );
        },
      );

      const deleteOption = this.createContextMenuItem("Delete", () => {
        this._removeOperation(selectedLocation);
        this.renderFn();
      });

      if (!selectedOperation.isMeasurement) {
        // Note: X has a special symbol that doesn't allow for adjoint or args.
        // In the future, we may want to create context menus off of host elements rather
        // than gate elements. Then we can generalize this exception.
        if (selectedOperation.gate != "X") {
          contextMenu.appendChild(adjointOption);
        }
        contextMenu.appendChild(addControlOption);
        if (removeControlOption) {
          contextMenu.appendChild(removeControlOption);
        }
        if (selectedOperation.gate != "X") {
          contextMenu.appendChild(promptArgOption);
        }
      }
      contextMenu.appendChild(deleteOption);
      document.body.appendChild(contextMenu);

      document.addEventListener(
        "click",
        () => {
          if (document.body.contains(contextMenu)) {
            document.body.removeChild(contextMenu);
          }
        },
        { once: true },
      );
    });
  }

  /**
   * Create a custom prompt element
   * @param message - The message to display in the prompt
   * @param callback - The callback function to handle the user input
   * @param defaultValue - The default value to display in the input element
   */
  _createCustomPrompt(
    message: string,
    callback: (input: string | null) => void,
    defaultValue: string = "",
  ) {
    // Create the prompt overlay
    const overlay = document.createElement("div");
    overlay.classList.add("custom-prompt-overlay");

    // Create the prompt container
    const promptContainer = document.createElement("div");
    promptContainer.classList.add("custom-prompt-container");

    // Create the message element
    const messageElem = document.createElement("div");
    messageElem.classList.add("custom-prompt-message");
    messageElem.textContent = message;

    // Create the input element
    const inputElem = document.createElement("input");
    inputElem.classList.add("custom-prompt-input");
    inputElem.type = "text";
    inputElem.value = defaultValue;

    // Create the buttons container
    const buttonsContainer = document.createElement("div");
    buttonsContainer.classList.add("custom-prompt-buttons");

    // Create the OK button
    const okButton = document.createElement("button");
    okButton.classList.add("custom-prompt-button");
    okButton.textContent = "OK";
    okButton.addEventListener("click", () => {
      callback(inputElem.value);
      document.body.removeChild(overlay);
    });

    // Create the Cancel button
    const cancelButton = document.createElement("button");
    cancelButton.classList.add("custom-prompt-button");
    cancelButton.textContent = "Cancel";
    cancelButton.addEventListener("click", () => {
      callback(null);
      document.body.removeChild(overlay);
    });

    // Append elements to the prompt container
    buttonsContainer.appendChild(okButton);
    buttonsContainer.appendChild(cancelButton);
    promptContainer.appendChild(messageElem);
    promptContainer.appendChild(inputElem);
    promptContainer.appendChild(buttonsContainer);
    overlay.appendChild(promptContainer);

    // Append the overlay to the body
    document.body.appendChild(overlay);
  }

  /**
   * Add events for circuit objects in the circuit
   */
  _addToolboxElementsEvents() {
    const elems = this._toolboxElems();
    elems.forEach((elem) => {
      elem.addEventListener("mousedown", (ev: MouseEvent) => {
        if (ev.button !== 0) return;
        this.container.classList.add("moving");
        this.dropzoneLayer.style.display = "block";
        const type = elem.getAttribute("data-type");
        if (type == null) return;
        this.selectedOperation = defaultGateDictionary[type];
        this.createGhostElement(ev);
      });
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
        const sourceLocation = this._getLocation(this.selectedOperation);

        if (sourceLocation == null) {
          // Add a new operation from the toolbox
          this._addOperation(this.selectedOperation, targetLoc, targetWire);
          // const newOperation = this._addOperation(this.selectedOperation, targetLoc, targetWire);
          // if (newOperation) {
          //     this._moveY(targetWire, newOperation, this.wireData.length);
          // }
        } else if (sourceLocation && this.selectedWire != null) {
          if (ev.ctrlKey) {
            this._addOperation(this.selectedOperation, targetLoc, targetWire);
          } else {
            const newOperation = this._moveX(
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
                parentOperation.targets = this._targets(parentOperation);
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
        this._findAndRemoveOperations(check);
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

    const indexes = this._indexes(location);
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

    const indexes = this._indexes(location);
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

    const index = this._lastIndex(location);
    const operationParent = this._findParentArray(location);

    if (operationParent == null || index == null) return null;

    return operationParent[index];
  }

  _findAndRemoveOperations(pred: (op: Operation) => boolean): void {
    const originalOperations = cloneDeep(this.operations);

    const inPlaceFilter = (
      ops: Operation[],
      pred: (op: Operation) => boolean,
    ) => {
      let i = 0;
      while (i < ops.length) {
        if (!pred(ops[i])) {
          ops.splice(i, 1);
        } else {
          i++;
        }
      }
    };

    const recursivePred = (op: Operation) => {
      if (pred(op)) return true;
      if (op.children) {
        inPlaceFilter(op.children, (child) => !recursivePred(child));
      }
      return false;
    };

    inPlaceFilter(this.operations, (op) => !recursivePred(op));
    if (isEqual(originalOperations, this.operations) === false) this.renderFn();
  }

  /**************************
   *  Circuit Manipulation  *
   **************************/

  /**
   * Remove an operation from the circuit
   */
  _removeOperation(sourceLocation: string) {
    const sourceOperation = this._findOperation(sourceLocation);
    const sourceOperationParent = this._findParentArray(sourceLocation);

    if (sourceOperation == null || sourceOperationParent == null) return null;

    // Delete sourceOperation
    if (sourceOperation.dataAttributes === undefined) {
      sourceOperation.dataAttributes = { removed: "true" };
    } else {
      sourceOperation.dataAttributes["removed"] = "true";
    }
    const indexToRemove = sourceOperationParent.findIndex(
      (operation) =>
        operation.dataAttributes && operation.dataAttributes["removed"],
    );
    sourceOperationParent.splice(indexToRemove, 1);

    if (sourceOperation.isMeasurement) {
      this._removeMeasurementLines(sourceOperation);
    }
  }

  /**
   * Removes all measurement lines of a measure from the circuit and adjust the cIds of the other measurements
   */
  _removeMeasurementLines(sourceOperation: Operation) {
    for (const target of sourceOperation.targets) {
      const qubit = this.qubits[target.qId];
      if (qubit.numChildren != undefined && target.cId != undefined) {
        for (const op of this.operations) {
          if (op.controls) {
            for (const control of op.controls) {
              if (
                control.qId === target.qId &&
                control.cId &&
                control.cId > target.cId
              ) {
                control.cId--;
              }
            }
          }
          for (const targetReg of op.targets) {
            if (
              targetReg.qId === target.qId &&
              targetReg.cId &&
              targetReg.cId > target.cId
            ) {
              targetReg.cId--;
            }
          }
        }
        qubit.numChildren--;
      }
    }
  }

  /**
   * Move an operation horizontally
   */
  _moveX = (
    sourceLocation: string,
    targetLocation: string,
    targetWire: number,
  ): Operation | null => {
    const sourceOperation = this._findOperation(sourceLocation);
    if (sourceLocation === targetLocation) return sourceOperation;
    const sourceOperationParent = this._findParentArray(sourceLocation);
    const targetOperationParent = this._findParentArray(targetLocation);
    const targetLastIndex = this._lastIndex(targetLocation);

    if (
      targetOperationParent == null ||
      targetLastIndex == null ||
      sourceOperation == null ||
      sourceOperationParent == null
    )
      return null;

    // Insert sourceOperation to target last index
    const newSourceOperation: Operation = JSON.parse(
      JSON.stringify(sourceOperation),
    );
    if (newSourceOperation.isMeasurement) {
      this._addMeasurementLine(newSourceOperation, targetWire);
    }
    targetOperationParent.splice(targetLastIndex, 0, newSourceOperation);

    // Delete sourceOperation
    if (sourceOperation.dataAttributes === undefined) {
      sourceOperation.dataAttributes = { removed: "true" };
    } else {
      sourceOperation.dataAttributes["removed"] = "true";
    }
    const indexToRemove = sourceOperationParent.findIndex(
      (operation) =>
        operation.dataAttributes && operation.dataAttributes["removed"],
    );
    sourceOperationParent.splice(indexToRemove, 1);

    if (sourceOperation.isMeasurement) {
      this._removeMeasurementLines(sourceOperation);
    }

    return newSourceOperation;
  };

  /**
   * Add an operation into the circuit
   */
  _addOperation = (
    sourceOperation: Operation,
    targetLocation: string,
    targetWire: number,
  ): Operation | null => {
    const targetOperationParent = this._findParentArray(targetLocation);
    const targetLastIndex = this._lastIndex(targetLocation);

    if (
      targetOperationParent == null ||
      targetLastIndex == null ||
      sourceOperation == null
    )
      return null;

    // Insert sourceOperation to target last index
    const newSourceOperation: Operation = JSON.parse(
      JSON.stringify(sourceOperation),
    );
    if (newSourceOperation.isMeasurement) {
      this._addMeasurementLine(newSourceOperation, targetWire);
    } else {
      newSourceOperation.targets = [{ qId: targetWire, type: 0 }];
    }
    targetOperationParent.splice(targetLastIndex, 0, newSourceOperation);

    return newSourceOperation;
  };

  /**
   * Add a measurement line to the circuit and attach the source operation
   */
  _addMeasurementLine = (
    sourceOperation: Operation,
    targetQubitWire: number,
  ) => {
    const newNumChildren = this.qubits[targetQubitWire].numChildren
      ? this.qubits[targetQubitWire].numChildren + 1
      : 1;
    this.qubits[targetQubitWire].numChildren = newNumChildren;
    sourceOperation.targets = [
      { qId: targetQubitWire, type: 1, cId: newNumChildren - 1 },
    ];
    sourceOperation.controls = [{ qId: targetQubitWire, type: 0 }];
  };

  /**
   * Add a control to the specified operation on the given wire index
   *
   * @param op - The operation to which the control will be added
   * @param wireIndex - The index of the wire where the control will be added
   */
  addControl(op: Operation, wireIndex: number) {
    if (!op.controls) {
      op.controls = [];
    }
    const existingControl = op.controls.find(
      (control) => control.qId === wireIndex,
    );
    if (!existingControl) {
      op.controls.push({
        qId: wireIndex,
        type: 0,
      });
      op.controls.sort((a, b) => a.qId - b.qId);
      op.isControlled = true;
      this.renderFn();
    }
    this.selectedOperation = null;
    this.container.classList.remove("adding-control");
  }

  /**
   * Remove a control from the specified operation on the given wire index
   *
   * @param op - The operation from which the control will be removed
   * @param wireIndex - The index of the wire where the control will be removed
   */
  removeControl(op: Operation, wireIndex: number) {
    if (op.controls) {
      const controlIndex = op.controls.findIndex(
        (control) => control.qId === wireIndex,
      );
      if (controlIndex !== -1) {
        op.controls.splice(controlIndex, 1);
        if (op.controls.length === 0) {
          op.isControlled = false;
        }
        this.renderFn();
      }
    }
    this.selectedOperation = null;
    this.container.classList.remove("removing-control");
  }

  /**
   * Move an operation vertically by changing its controls and targets
   */
  // ToDo: this should be repurposed to move a multi-target operation to a different wire
  _moveY = (
    targetWire: number,
    operation: Operation,
    totalWires: number,
  ): Operation => {
    if (!operation.isMeasurement) {
      this._offsetRecursively(operation, targetWire, totalWires);
    }
    return operation;
  };

  /*****************
   *     Misc.     *
   *****************/

  /**
   * Create a context menu item
   * @param text - The text to display in the menu item
   * @param onClick - The function to call when the menu item is clicked
   * @returns The created menu item element
   */
  createContextMenuItem(text: string, onClick: () => void): HTMLDivElement {
    const menuItem = document.createElement("div");
    menuItem.classList.add("context-menu-option");
    menuItem.textContent = text;
    menuItem.addEventListener("click", onClick);
    return menuItem;
  }

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
        dropzone.addEventListener("click", () => {
          if (this.selectedOperation != null) {
            this.addControl(this.selectedOperation, wireIndex);
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
      dropzone.addEventListener("click", () => {
        if (this.selectedOperation != null) {
          this.removeControl(this.selectedOperation, control.qId);
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
   * Gets the location of an operation, if it has one
   */
  _getLocation(operation: Operation): string | null {
    if (operation.dataAttributes == null) return null;
    return operation.dataAttributes["location"];
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

  /**
   * Recursively change object controls and targets
   */
  _offsetRecursively(
    operation: Operation,
    wireOffset: number,
    totalWires: number,
  ): Operation {
    // Offset all targets by offsetY value
    if (operation.targets) {
      operation.targets.forEach((target) => {
        target.qId = this._circularMod(target.qId, wireOffset, totalWires);
        if (target.cId)
          target.cId = this._circularMod(target.cId, wireOffset, totalWires);
      });
    }

    // Offset all controls by offsetY value
    if (operation.controls) {
      operation.controls.forEach((control) => {
        control.qId = this._circularMod(control.qId, wireOffset, totalWires);
        if (control.cId)
          control.cId = this._circularMod(control.qId, wireOffset, totalWires);
      });
    }

    // Offset recursively through all children
    if (operation.children) {
      operation.children.forEach((child) =>
        this._offsetRecursively(child, wireOffset, totalWires),
      );
    }

    return operation;
  }

  /**
   * Find targets of an operation by recursively walkthrough all of its children controls and targets
   * i.e. Gate Foo contains gate H and gate RX.
   *      qIds of Gate H is 1
   *      qIds of Gate RX is 1, 2
   *      This should return [{qId: 1}, {qId: 2}]
   */
  _targets(operation: Operation): Register[] | [] {
    const _recurse = (operation: Operation) => {
      registers.push(...operation.targets);
      if (operation.controls) {
        registers.push(...operation.controls);
        // If there is more children, keep adding more to registers
        if (operation.children) {
          for (const child of operation.children) {
            _recurse(child);
          }
        }
      }
    };

    const registers: Register[] = [];
    if (operation.children == null) return [];

    // Recursively walkthrough all children to populate registers
    for (const child of operation.children) {
      _recurse(child);
    }

    // Extract qIds from array of object
    // i.e. [{qId: 0}, {qId: 1}, {qId: 1}] -> [0, 1, 1]
    const qIds = registers.map((register) => register.qId);
    const uniqueQIds = Array.from(new Set(qIds));

    // Transform array of numbers into array of qId object
    // i.e. [0, 1] -> [{qId: 0}, {qId: 1}]
    return uniqueQIds.map((qId) => ({
      qId,
      type: 0,
    }));
  }

  /**
   * This modulo function always returns positive value based on total
   * i.e: value=0, offset=-1, total=4 returns 3 instead of -1
   */
  _circularMod(value: number, offset: number, total: number): number {
    return (((value + offset) % total) + total) % total;
  }

  /**
   * Split location into an array of indexes
   */
  _indexes(location: string): number[] {
    return location !== ""
      ? location.split("-").map((segment) => parseInt(segment))
      : [];
  }

  /**
   * Get the last index of location
   * i.e: location = "0-1-2", _lastIndex will return 2
   */
  _lastIndex(location: string): number | undefined {
    return this._indexes(location).pop();
  }
}

export { extensionEvents };
