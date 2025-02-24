// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Operation } from "./circuit";
import { CircuitEvents } from "./events";
import { RegisterType } from "./register";
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
    circuitEvents.operations,
    sourceLocation,
  );
  if (sourceLocation === targetLocation) return sourceOperation;
  const sourceOperationParent = findParentArray(
    circuitEvents.operations,
    sourceLocation,
  );
  const targetOperationParent = findParentArray(
    circuitEvents.operations,
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
    newSourceOperation.targets = [
      { qId: targetWire, type: RegisterType.Qubit },
    ];
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
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceOperation The operation to be added.
 * @param targetOperationParent The parent array where the operation will be added.
 * @param targetLastIndex The index within the parent array where the operation will be added.
 * @param targetWire The wire index to add the operation to.
 * @param insertNewColumn Whether to insert a new column when adding the operation.
 * @returns The added operation.
 */
const _addOp = (
  circuitEvents: CircuitEvents,
  sourceOperation: Operation,
  targetOperationParent: Operation[][],
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
    targetOperationParent[colIndex] = [];
  }
  if (insertNewColumn) {
    targetOperationParent.splice(colIndex, 0, [newSourceOperation]);
  } else {
    targetOperationParent[colIndex].splice(opIndex, 0, newSourceOperation);
  }
  return newSourceOperation;
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
