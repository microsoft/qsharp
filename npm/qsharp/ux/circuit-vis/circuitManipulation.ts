// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { ComponentGrid, Operation } from "./circuit";
import { CircuitEvents } from "./events";
import {
  findOperation,
  findParentArray,
  findParentOperation,
  getGateTargets,
  locationStringToIndexes,
} from "./utils";

/**
 * Move an operation in the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the source operation.
 * @param targetLocation The location string of the target position.
 * @param sourceWire The wire index of the source operation.
 * @param targetWire The wire index to move the operation to.
 * @param movingControl Whether the operation is being moved as a control.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @returns The moved operation or null if the move was unsuccessful.
 */
const moveOperation = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
  targetLocation: string,
  sourceWire: number,
  targetWire: number,
  movingControl: boolean,
  insertNewColumn: boolean = false,
): Operation | null => {
  const sourceOperation = _moveX(
    circuitEvents,
    sourceLocation,
    targetLocation,
    targetWire,
    insertNewColumn,
  );

  if (sourceOperation == null) return null;

  // Update sourceOperation targets and controls
  _moveY(
    circuitEvents,
    sourceOperation,
    sourceLocation,
    sourceWire,
    targetWire,
    movingControl,
  );

  return sourceOperation;
};

/**
 * Move an operation horizontally.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the source operation.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to move the operation to.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @returns The moved operation or null if the move was unsuccessful.
 */
const _moveX = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
  targetLocation: string,
  targetWire: number,
  insertNewColumn: boolean = false,
): Operation | null => {
  const sourceOperation = findOperation(
    circuitEvents.operationGrid,
    sourceLocation,
  );
  if (!insertNewColumn && sourceLocation === targetLocation)
    return sourceOperation;
  const sourceOperationParent = findParentArray(
    circuitEvents.operationGrid,
    sourceLocation,
  );
  const targetOperationParent = findParentArray(
    circuitEvents.operationGrid,
    targetLocation,
  );
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (
    targetOperationParent == null ||
    targetLastIndex == null ||
    sourceOperation == null ||
    sourceOperationParent == null
  )
    return null;

  // Insert sourceOperation to target last index
  const newSourceOperation = _addOp(
    circuitEvents,
    sourceOperation,
    targetOperationParent,
    targetLastIndex,
    targetWire,
    insertNewColumn,
  );

  // Delete sourceOperation
  _removeOp(circuitEvents, sourceOperation, sourceOperationParent);

  return newSourceOperation;
};

/**
 * Move an operation vertically by changing its controls and targets.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be moved.
 * @param sourceLocation The location string of the source operation.
 * @param sourceWire The wire index of the source operation.
 * @param targetWire The wire index to move the operation to.
 * @param movingControl Whether the operation is being moved as a control.
 */
const _moveY = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  sourceLocation: string,
  sourceWire: number,
  targetWire: number,
  movingControl: boolean,
): void => {
  if (sourceOperation.isMeasurement) {
    _removeMeasurementLines(circuitEvents, sourceOperation);
    _addMeasurementLine(circuitEvents, sourceOperation, targetWire);
  } else {
    if (movingControl) {
      sourceOperation.controls?.forEach((control) => {
        if (control.qubit === sourceWire) {
          control.qubit = targetWire;
        }
      });
      sourceOperation.controls = sourceOperation.controls?.sort(
        (a, b) => a.qubit - b.qubit,
      );
    } else {
      sourceOperation.targets = [{ qubit: targetWire }];
    }
  }

  // Update parent operation targets
  const parentOperation = findParentOperation(
    circuitEvents.operationGrid,
    sourceLocation,
  );
  if (parentOperation) {
    parentOperation.targets = getGateTargets(parentOperation);
  }
};

/**
 * Add an operation into the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be added.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to add the operation to.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @returns The added operation or null if the addition was unsuccessful.
 */
const addOperation = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  targetLocation: string,
  targetWire: number,
  insertNewColumn: boolean = false,
): Operation | null => {
  const targetOperationParent = findParentArray(
    circuitEvents.operationGrid,
    targetLocation,
  );
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (
    targetOperationParent == null ||
    targetLastIndex == null ||
    sourceOperation == null
  )
    return null;

  const newSourceOperation = _addOp(
    circuitEvents,
    sourceOperation,
    targetOperationParent,
    targetLastIndex,
    targetWire,
    insertNewColumn,
  );
  if (!newSourceOperation.isMeasurement) {
    newSourceOperation.targets = [{ qubit: targetWire }];
  }

  return newSourceOperation;
};

/**
 * Remove an operation from the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the operation to be removed.
 */
const removeOperation = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
) => {
  const sourceOperation = findOperation(
    circuitEvents.operationGrid,
    sourceLocation,
  );
  const sourceOperationParent = findParentArray(
    circuitEvents.operationGrid,
    sourceLocation,
  );

  if (sourceOperation == null || sourceOperationParent == null) return null;

  // Delete sourceOperation
  _removeOp(circuitEvents, sourceOperation, sourceOperationParent);
};

/**
 * Find and remove operations in-place based on a predicate function.
 *
 * @param componentGrid The grid of components to search through.
 * @param pred The predicate function to determine which operations to remove.
 */
const findAndRemoveOperations = (
  componentGrid: ComponentGrid,
  pred: (op: Operation) => boolean,
) => {
  const inPlaceFilter = (
    grid: ComponentGrid,
    pred: (op: Operation) => boolean,
  ) => {
    let i = 0;
    while (i < grid.length) {
      let j = 0;
      while (j < grid[i].components.length) {
        if (!pred(grid[i].components[j])) {
          grid[i].components.splice(j, 1);
        } else {
          j++;
        }
      }
      if (grid[i].components.length === 0) {
        grid.splice(i, 1);
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

  inPlaceFilter(componentGrid, (op) => !recursivePred(op));
};

/**
 * Add a control to the specified operation on the given wire index.
 *
 * @param op The operation to which the control will be added.
 * @param wireIndex The index of the wire where the control will be added.
 * @returns True if the control was added, false if it already existed.
 */
const addControl = (op: Operation, wireIndex: number): boolean => {
  if (!op.controls) {
    op.controls = [];
  }
  const existingControl = op.controls.find(
    (control) => control.qubit === wireIndex,
  );
  if (!existingControl) {
    op.controls.push({ qubit: wireIndex });
    op.controls.sort((a, b) => a.qubit - b.qubit);
    return true;
  }
  return false;
};

/**
 * Remove a control from the specified operation on the given wire index.
 *
 * @param op The operation from which the control will be removed.
 * @param wireIndex The index of the wire where the control will be removed.
 * @returns True if the control was removed, false if it did not exist.
 */
const removeControl = (op: Operation, wireIndex: number): boolean => {
  if (op.controls) {
    const controlIndex = op.controls.findIndex(
      (control) => control.qubit === wireIndex,
    );
    if (controlIndex !== -1) {
      op.controls.splice(controlIndex, 1);
      return true;
    }
  }
  return false;
};

/**
 * Add an operation to the circuit at the specified location.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be added.
 * @param targetOperationParent The parent grid where the operation will be added.
 * @param targetLastIndex The index within the parent array where the operation will be added.
 * @param targetWire The wire index to add the operation to.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @returns The added operation.
 */
const _addOp = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  targetOperationParent: ComponentGrid,
  targetLastIndex: [number, number],
  targetWire: number,
  insertNewColumn: boolean = false,
): Operation => {
  const newSourceOperation: Operation = JSON.parse(
    JSON.stringify(sourceOperation),
  );
  if (newSourceOperation.isMeasurement) {
    _addMeasurementLine(circuitEvents, newSourceOperation, targetWire);
  }
  const [colIndex, opIndex] = targetLastIndex;
  if (targetOperationParent[colIndex] == null) {
    targetOperationParent[colIndex] = { components: [] };
  }
  if (insertNewColumn) {
    targetOperationParent.splice(colIndex, 0, {
      components: [newSourceOperation],
    });
  } else {
    targetOperationParent[colIndex].components.splice(
      opIndex,
      0,
      newSourceOperation,
    );
  }
  return newSourceOperation;
};

/**
 * Remove an operation from the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be removed.
 * @param sourceOperationParent The parent grid from which the operation will be removed.
 */
const _removeOp = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  sourceOperationParent: ComponentGrid,
) => {
  if (sourceOperation.dataAttributes === undefined) {
    sourceOperation.dataAttributes = { removed: "true" };
  } else {
    sourceOperation.dataAttributes["removed"] = "true";
  }

  // Find and remove the operation in sourceOperationParent
  for (let colIndex = 0; colIndex < sourceOperationParent.length; colIndex++) {
    const col = sourceOperationParent[colIndex];
    const indexToRemove = col.components.findIndex(
      (operation) =>
        operation.dataAttributes && operation.dataAttributes["removed"],
    );
    if (indexToRemove !== -1) {
      col.components.splice(indexToRemove, 1);
      if (col.components.length === 0) {
        sourceOperationParent.splice(colIndex, 1);
      }
      break;
    }
  }

  if (sourceOperation.isMeasurement) {
    _removeMeasurementLines(circuitEvents, sourceOperation);
  }
};

/**
 * Add a measurement line to the circuit and attach the source operation.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to which the measurement line will be added.
 * @param targetQubitWire The wire index to add the measurement line to.
 */
const _addMeasurementLine = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  targetQubitWire: number,
) => {
  const newNumResults = circuitEvents.qubits[targetQubitWire].numResults
    ? circuitEvents.qubits[targetQubitWire].numResults + 1
    : 1;
  circuitEvents.qubits[targetQubitWire].numResults = newNumResults;
  sourceOperation.targets = [
    {
      qubit: targetQubitWire,
      result: newNumResults - 1,
    },
  ];
  sourceOperation.controls = [{ qubit: targetQubitWire }];
};

/**
 * Removes all measurement lines of a measure from the circuit and adjust the cIds of the other measurements.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation from which the measurement lines will be removed.
 */
const _removeMeasurementLines = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
) => {
  for (const target of sourceOperation.targets) {
    const qubit = circuitEvents.qubits[target.qubit];
    if (qubit.numResults != undefined && target.result != undefined) {
      for (const col of circuitEvents.operationGrid) {
        for (const op of col.components) {
          if (op.controls) {
            for (const control of op.controls) {
              if (
                control.qubit === target.qubit &&
                control.result &&
                control.result > target.result
              ) {
                control.result--;
              }
            }
          }
          for (const targetReg of op.targets) {
            if (
              targetReg.qubit === target.qubit &&
              targetReg.result &&
              targetReg.result > target.result
            ) {
              targetReg.result--;
            }
          }
        }
      }
      qubit.numResults--;
    }
  }
};

export {
  moveOperation,
  addOperation,
  removeOperation,
  findAndRemoveOperations,
  addControl,
  removeControl,
};
