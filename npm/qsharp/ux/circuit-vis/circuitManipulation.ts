// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import { CircuitEvents } from "./events";
import { Register, RegisterType } from "./register";
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
  const originalOperation = findOperation(
    circuitEvents.operations,
    sourceLocation,
  );

  if (originalOperation == null) return null;

  // Create a deep copy of the source operation
  const newSourceOperation: Operation = JSON.parse(
    JSON.stringify(originalOperation),
  );

  // Update operation's targets and controls
  _moveY(
    circuitEvents,
    newSourceOperation,
    sourceLocation,
    sourceWire,
    targetWire,
    movingControl,
  );

  // Move horizontally
  _moveX(
    circuitEvents,
    newSourceOperation,
    originalOperation,
    targetLocation,
    insertNewColumn,
  );

  const sourceOperationParent = findParentArray(
    circuitEvents.operations,
    sourceLocation,
  );
  if (sourceOperationParent == null) return null;
  _removeOp(circuitEvents, originalOperation, sourceOperationParent);

  return newSourceOperation;
};

/**
 * Move an operation horizontally.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be moved.
 * @param originalOperation The original source operation to be ignored during the check for existing operations.
 * @param targetLocation The location string of the target position.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 */
const _moveX = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  originalOperation: Operation,
  targetLocation: string,
  insertNewColumn: boolean = false,
) => {
  const targetOperationParent = findParentArray(
    circuitEvents.operations,
    targetLocation,
  );

  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (targetOperationParent == null || targetLastIndex == null) return;

  // Insert sourceOperation to target last index
  _addOp(
    sourceOperation,
    targetOperationParent,
    targetLastIndex,
    insertNewColumn,
    originalOperation,
  );
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
    _addMeasurementLine(circuitEvents, sourceOperation, targetWire);
  } else if (movingControl) {
    sourceOperation.controls?.forEach((control) => {
      if (control.qId === sourceWire) {
        control.qId = targetWire;
      }
    });
    sourceOperation.controls = sourceOperation.controls?.sort(
      (a, b) => a.qId - b.qId,
    );
  } else {
    sourceOperation.targets = [{ qId: targetWire, type: RegisterType.Qubit }];
  }

  // Update parent operation targets
  const parentOperation = findParentOperation(
    circuitEvents.operations,
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
    circuitEvents.operations,
    targetLocation,
  );
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (targetOperationParent == null || targetLastIndex == null) return null;
  // Create a deep copy of the source operation
  const newSourceOperation: Operation = JSON.parse(
    JSON.stringify(sourceOperation),
  );

  if (sourceOperation.isMeasurement) {
    _addMeasurementLine(circuitEvents, newSourceOperation, targetWire);
  } else {
    newSourceOperation.targets = [
      { qId: targetWire, type: RegisterType.Qubit },
    ];
  }

  _addOp(
    newSourceOperation,
    targetOperationParent,
    targetLastIndex,
    insertNewColumn,
  );

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
    circuitEvents.operations,
    sourceLocation,
  );
  const sourceOperationParent = findParentArray(
    circuitEvents.operations,
    sourceLocation,
  );

  if (sourceOperation == null || sourceOperationParent == null) return null;

  // Delete sourceOperation
  _removeOp(circuitEvents, sourceOperation, sourceOperationParent);
};

/**
 * Find and remove operations in-place based on a predicate function.
 *
 * @param operations The array of operations to search through.
 * @param pred The predicate function to determine which operations to remove.
 */
const findAndRemoveOperations = (
  operations: Operation[][],
  pred: (op: Operation) => boolean,
) => {
  const inPlaceFilter = (
    ops: Operation[][],
    pred: (op: Operation) => boolean,
  ) => {
    let i = 0;
    while (i < ops.length) {
      let j = 0;
      while (j < ops[i].length) {
        if (!pred(ops[i][j])) {
          ops[i].splice(j, 1);
        } else {
          j++;
        }
      }
      if (ops[i].length === 0) {
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

  inPlaceFilter(operations, (op) => !recursivePred(op));
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
    (control) => control.qId === wireIndex,
  );
  if (!existingControl) {
    op.controls.push({
      qId: wireIndex,
      type: RegisterType.Qubit,
    });
    op.controls.sort((a, b) => a.qId - b.qId);
    op.isControlled = true;
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
      (control) => control.qId === wireIndex,
    );
    if (controlIndex !== -1) {
      op.controls.splice(controlIndex, 1);
      if (op.controls.length === 0) {
        op.isControlled = false;
      }
      return true;
    }
  }
  return false;
};

/**
 * Add an operation to the circuit at the specified location.
 *
 * @param sourceOperation The operation to be added.
 * @param targetOperationParent The parent array where the operation will be added.
 * @param targetLastIndex The index within the parent array where the operation will be added.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @param originalOperation The original source operation to be ignored during the check for existing operations.
 */
const _addOp = (
  sourceOperation: Operation,
  targetOperationParent: Operation[][],
  targetLastIndex: [number, number],
  insertNewColumn: boolean = false,
  originalOperation: Operation | null = null,
) => {
  const [colIndex, opIndex] = targetLastIndex;
  if (targetOperationParent[colIndex] == null) {
    targetOperationParent[colIndex] = [];
  }

  insertNewColumn =
    insertNewColumn || _isClassicallyControlled(sourceOperation);

  // Check if there are any existing operations in the target
  // column within the wire range of the new operation
  if (!insertNewColumn) {
    const [minTarget, maxTarget] = _getMinMaxRegIdx(sourceOperation);
    for (const op of targetOperationParent[colIndex]) {
      if (op === originalOperation) continue;

      const [opMinTarget, opMaxTarget] = _getMinMaxRegIdx(op);
      if (
        (opMinTarget >= minTarget && opMinTarget <= maxTarget) ||
        (opMaxTarget >= minTarget && opMaxTarget <= maxTarget) ||
        (minTarget >= opMinTarget && minTarget <= opMaxTarget) ||
        (maxTarget >= opMinTarget && maxTarget <= opMaxTarget)
      ) {
        insertNewColumn = true;
        break;
      }
    }
  }

  if (insertNewColumn) {
    targetOperationParent.splice(colIndex, 0, [sourceOperation]);
  } else {
    targetOperationParent[colIndex].splice(opIndex, 0, sourceOperation);
  }
  return sourceOperation;
};

/**
 * Get the minimum and maximum register indices for a given operation.
 * Based on getMinMaxRegIdx in process.ts, but without the maxQId.
 *
 * @param operation The operation for which to get the register indices.
 * @returns A tuple containing the minimum and maximum register indices.
 */
const _getMinMaxRegIdx = (operation: Operation): [number, number] => {
  const { targets, controls } = operation;
  const ctrls: Register[] = controls || [];
  const qRegs: Register[] = [...ctrls, ...targets].filter(
    ({ type }) => (type || RegisterType.Qubit) === RegisterType.Qubit,
  );
  if (qRegs.length === 0) return [-1, -1];
  const qRegIdxList: number[] = qRegs.map(({ qId }) => qId);
  // Pad the contiguous range of registers that it covers.
  const minRegIdx: number = Math.min(...qRegIdxList);
  const maxRegIdx: number = Math.max(...qRegIdxList);

  return [minRegIdx, maxRegIdx];
};

/**
 * Check if an operation is classically controlled.
 *
 * @param operation The operation for which to get the register indices.
 * @returns True if the operation is classically controlled, false otherwise.
 */
const _isClassicallyControlled = (operation: Operation): boolean => {
  if (operation.controls === undefined) return false;
  const clsControl = operation.controls.find(
    ({ type }) => (type || RegisterType.Qubit) === RegisterType.Classical,
  );
  return clsControl !== undefined;
};

/**
 * Remove an operation from the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be removed.
 * @param sourceOperationParent The parent array from which the operation will be removed.
 */
const _removeOp = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  sourceOperationParent: Operation[][],
) => {
  if (sourceOperation.dataAttributes === undefined) {
    sourceOperation.dataAttributes = { removed: "true" };
  } else {
    sourceOperation.dataAttributes["removed"] = "true";
  }

  // Find and remove the operation in sourceOperationParent
  for (let colIndex = 0; colIndex < sourceOperationParent.length; colIndex++) {
    const col = sourceOperationParent[colIndex];
    const indexToRemove = col.findIndex(
      (operation) =>
        operation.dataAttributes && operation.dataAttributes["removed"],
    );
    if (indexToRemove !== -1) {
      col.splice(indexToRemove, 1);
      if (col.length === 0) {
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
  const newNumChildren = circuitEvents.qubits[targetQubitWire].numChildren
    ? circuitEvents.qubits[targetQubitWire].numChildren + 1
    : 1;
  circuitEvents.qubits[targetQubitWire].numChildren = newNumChildren;
  sourceOperation.targets = [
    {
      qId: targetQubitWire,
      type: RegisterType.Classical,
      cId: newNumChildren - 1,
    },
  ];
  sourceOperation.controls = [
    { qId: targetQubitWire, type: RegisterType.Qubit },
  ];
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
    const qubit = circuitEvents.qubits[target.qId];
    if (qubit.numChildren != undefined && target.cId != undefined) {
      for (const col of circuitEvents.operations) {
        for (const op of col) {
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
      }
      qubit.numChildren--;
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
