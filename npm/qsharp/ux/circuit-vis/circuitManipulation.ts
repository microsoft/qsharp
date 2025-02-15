// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import { CircuitEvents } from "./events";
import { locationStringToIndexes } from "./utils";

/**
 * Move an operation horizontally.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the source operation.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to move the operation to.
 * @returns The moved operation or null if the move was unsuccessful.
 */
const moveX = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
  targetLocation: string,
  targetWire: number,
): Operation | null => {
  const sourceOperation = circuitEvents._findOperation(sourceLocation);
  if (sourceLocation === targetLocation) return sourceOperation;
  const sourceOperationParent = circuitEvents._findParentArray(sourceLocation);
  const targetOperationParent = circuitEvents._findParentArray(targetLocation);
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

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
    _addMeasurementLine(circuitEvents, newSourceOperation, targetWire);
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
    _removeMeasurementLines(circuitEvents, sourceOperation);
  }

  return newSourceOperation;
};

// /**
//  * Move an operation vertically by changing its controls and targets
//  */
// // ToDo: this should be repurposed to move a multi-target operation to a different wire
// const _moveY = (
//   targetWire: number,
//   operation: Operation,
//   totalWires: number,
// ): Operation => {
//   if (!operation.isMeasurement) {
//     _offsetRecursively(operation, targetWire, totalWires);
//   }
//   return operation;
// };

// /**
//  * Recursively change object controls and targets
//  */
// const _offsetRecursively = (
//   operation: Operation,
//   wireOffset: number,
//   totalWires: number,
// ): Operation => {
//   // Offset all targets by offsetY value
//   if (operation.targets) {
//     operation.targets.forEach((target) => {
//       target.qId = _circularMod(target.qId, wireOffset, totalWires);
//       if (target.cId)
//         target.cId = _circularMod(target.cId, wireOffset, totalWires);
//     });
//   }

//   // Offset all controls by offsetY value
//   if (operation.controls) {
//     operation.controls.forEach((control) => {
//       control.qId = _circularMod(control.qId, wireOffset, totalWires);
//       if (control.cId)
//         control.cId = _circularMod(control.qId, wireOffset, totalWires);
//     });
//   }

//   // Offset recursively through all children
//   if (operation.children) {
//     operation.children.forEach((child) =>
//       _offsetRecursively(child, wireOffset, totalWires),
//     );
//   }

//   return operation;
// };

// /**
//  * This modulo function always returns positive value based on total
//  * i.e: value=0, offset=-1, total=4 returns 3 instead of -1
//  */
// const _circularMod = (value: number, offset: number, total: number): number => {
//   return (((value + offset) % total) + total) % total;
// };

/**
 * Add an operation into the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be added.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to add the operation to.
 * @returns The added operation or null if the addition was unsuccessful.
 */
const addOperation = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  targetLocation: string,
  targetWire: number,
): Operation | null => {
  const targetOperationParent = circuitEvents._findParentArray(targetLocation);
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

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
    _addMeasurementLine(circuitEvents, newSourceOperation, targetWire);
  } else {
    newSourceOperation.targets = [{ qId: targetWire, type: 0 }];
  }
  targetOperationParent.splice(targetLastIndex, 0, newSourceOperation);

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
  const sourceOperation = circuitEvents._findOperation(sourceLocation);
  const sourceOperationParent = circuitEvents._findParentArray(sourceLocation);

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
    _removeMeasurementLines(circuitEvents, sourceOperation);
  }
};

/**
 * Find and remove operations in-place based on a predicate function.
 *
 * @param operations The array of operations to search through.
 * @param pred The predicate function to determine which operations to remove.
 */
const findAndRemoveOperations = (
  operations: Operation[],
  pred: (op: Operation) => boolean,
) => {
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
      type: 0,
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
    { qId: targetQubitWire, type: 1, cId: newNumChildren - 1 },
  ];
  sourceOperation.controls = [{ qId: targetQubitWire, type: 0 }];
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
      for (const op of circuitEvents.operations) {
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
};

export {
  moveX,
  addOperation,
  removeOperation,
  findAndRemoveOperations,
  addControl,
  removeControl,
};
