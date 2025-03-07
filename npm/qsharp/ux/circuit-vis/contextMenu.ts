// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { removeControl, removeOperation } from "./circuitManipulation";
import { CircuitEvents } from "./events";
import { findGateElem, findOperation } from "./utils";

/**
 * Adds a context menu to a host element in the circuit visualization.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param hostElem The SVG element representing a gate component to which the context menu will be added.
 */
const addContextMenuToHostElem = (
  circuitEvents: CircuitEvents,
  hostElem: SVGGraphicsElement,
) => {
  hostElem?.addEventListener("contextmenu", (ev: MouseEvent) => {
    ev.preventDefault();

    // Remove any existing context menu
    const existingContextMenu = document.querySelector(".context-menu");
    if (existingContextMenu) {
      document.body.removeChild(existingContextMenu);
    }

    const gateElem = findGateElem(hostElem);
    if (!gateElem) return;
    const selectedLocation = gateElem.getAttribute("data-location");
    const selectedOperation = findOperation(
      circuitEvents.operationGrid,
      selectedLocation,
    );
    if (!selectedOperation || !selectedLocation) return;

    const contextMenu = document.createElement("div");
    contextMenu.classList.add("context-menu");
    contextMenu.style.top = `${ev.clientY}px`;
    contextMenu.style.left = `${ev.clientX}px`;
    contextMenu.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      e.stopPropagation();
    });
    contextMenu.addEventListener("mouseup", (e) => {
      e.preventDefault();
      e.stopPropagation();
    });

    const adjointOption = _createContextMenuItem("Toggle Adjoint", () => {
      selectedOperation.isAdjoint = !selectedOperation.isAdjoint;
      circuitEvents.renderFn();
    });

    const addControlOption = _createContextMenuItem("Add control", () => {
      circuitEvents._startAddingControl(selectedOperation);
    });

    let removeControlOption: HTMLDivElement | null = null;
    const dataWireStr = hostElem.getAttribute("data-wire");
    const dataWire = dataWireStr != null ? parseInt(dataWireStr) : null;
    const isControl =
      hostElem.classList.contains("control-dot") && dataWire != null;
    // The Remove Control option is different when targeting a control element directly.
    if (isControl) {
      removeControlOption = _createContextMenuItem("Remove control", () => {
        removeControl(selectedOperation, dataWire);
        circuitEvents.renderFn();
      });
    } else if (
      selectedOperation.controls &&
      selectedOperation.controls.length > 0
    ) {
      removeControlOption = _createContextMenuItem("Remove control", () => {
        circuitEvents._startRemovingControl(selectedOperation);
      });
    }

    const promptArgOption = _createContextMenuItem("Edit Argument", () => {
      _createCustomPrompt(
        "Argument for Gate:",
        (userInput) => {
          if (userInput !== null) {
            if (userInput == "") {
              selectedOperation.displayArgs = undefined;
            } else {
              selectedOperation.displayArgs = userInput;
            }
          }
          circuitEvents.renderFn();
        },
        selectedOperation.displayArgs,
      );
    });

    const deleteOption = _createContextMenuItem("Delete", () => {
      removeOperation(circuitEvents, selectedLocation);
      circuitEvents.renderFn();
    });

    if (isControl && removeControlOption) {
      contextMenu.appendChild(removeControlOption!);
    } else if (
      selectedOperation.isMeasurement ||
      selectedOperation.gate == "|0〉" ||
      selectedOperation.gate == "|1〉"
    ) {
      contextMenu.appendChild(deleteOption);
    } else if (selectedOperation.gate == "X") {
      contextMenu.appendChild(addControlOption);
      if (removeControlOption) {
        contextMenu.appendChild(removeControlOption);
      }
      contextMenu.appendChild(deleteOption);
    } else {
      contextMenu.appendChild(adjointOption);
      contextMenu.appendChild(addControlOption);
      if (removeControlOption) {
        contextMenu.appendChild(removeControlOption);
      }
      contextMenu.appendChild(promptArgOption);
      contextMenu.appendChild(deleteOption);
    }

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
};

/**
 * Create a context menu item
 * @param text - The text to display in the menu item
 * @param onClick - The function to call when the menu item is clicked
 * @returns The created menu item element
 */
const _createContextMenuItem = (
  text: string,
  onClick: () => void,
): HTMLDivElement => {
  const menuItem = document.createElement("div");
  menuItem.classList.add("context-menu-option");
  menuItem.textContent = text;
  menuItem.addEventListener("click", onClick);
  return menuItem;
};

/**
 * Create a custom prompt element
 * @param message - The message to display in the prompt
 * @param callback - The callback function to handle the user input
 * @param defaultValue - The default value to display in the input element
 */
const _createCustomPrompt = (
  message: string,
  callback: (input: string | null) => void,
  defaultValue: string = "",
) => {
  // Create the prompt overlay
  const overlay = document.createElement("div");
  overlay.classList.add("custom-prompt-overlay");
  overlay.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
  });

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
};

export { addContextMenuToHostElem };
