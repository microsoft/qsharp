// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Component, ComponentGrid, Measurement, Operation } from "./circuit";
import { CircuitEvents } from "./events";
import {
  findComponent,
  findParentArray,
  findParentComponent,
  getChildTargets,
  locationStringToIndexes,
} from "./utils";

/**
 * Move an component in the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the source component.
 * @param targetLocation The location string of the target position.
 * @param sourceWire The wire index of the source component.
 * @param targetWire The wire index to move the component to.
 * @param movingControl Whether the component is being moved as a control.
 * @param insertNewColumn Whether to insert a new column when adding the component.
 * @returns The moved component or null if the move was unsuccessful.
 */
const moveComponent = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
  targetLocation: string,
  sourceWire: number,
  targetWire: number,
  movingControl: boolean,
  insertNewColumn: boolean = false,
): Component | null => {
  const sourceComponent = _moveX(
    circuitEvents,
    sourceLocation,
    targetLocation,
    targetWire,
    insertNewColumn,
  );

  if (sourceComponent == null) return null;

  // Update sourceComponent targets and controls
  _moveY(
    circuitEvents,
    sourceComponent,
    sourceLocation,
    sourceWire,
    targetWire,
    movingControl,
  );

  return sourceComponent;
};

/**
 * Move a component horizontally.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the source component.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to move the component to.
 * @param insertNewColumn Whether to insert a new column when adding the component.
 * @returns The moved component or null if the move was unsuccessful.
 */
const _moveX = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
  targetLocation: string,
  targetWire: number,
  insertNewColumn: boolean = false,
): Component | null => {
  const sourceComponent = findComponent(
    circuitEvents.componentGrid,
    sourceLocation,
  );
  if (!insertNewColumn && sourceLocation === targetLocation)
    return sourceComponent;
  const sourceComponentParent = findParentArray(
    circuitEvents.componentGrid,
    sourceLocation,
  );
  const targetComponentParent = findParentArray(
    circuitEvents.componentGrid,
    targetLocation,
  );
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (
    targetComponentParent == null ||
    targetLastIndex == null ||
    sourceComponent == null ||
    sourceComponentParent == null
  )
    return null;

  // Insert sourceComponent to target last index
  const newSourceComponent = _addComp(
    circuitEvents,
    sourceComponent,
    targetComponentParent,
    targetLastIndex,
    targetWire,
    insertNewColumn,
  );

  // Delete sourceComponent
  _removeComp(circuitEvents, sourceComponent, sourceComponentParent);

  return newSourceComponent;
};

/**
 * Move an component vertically by changing its controls and targets.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceComponent The component to be moved.
 * @param sourceLocation The location string of the source component.
 * @param sourceWire The wire index of the source component.
 * @param targetWire The wire index to move the component to.
 * @param movingControl Whether the component is being moved as a control.
 */
const _moveY = (
  circuitEvents: CircuitEvents,
  sourceComponent: Component,
  sourceLocation: string,
  sourceWire: number,
  targetWire: number,
  movingControl: boolean,
): void => {
  if (sourceComponent.type === "Measurement") {
    _removeMeasurementLines(circuitEvents, sourceComponent);
    _addMeasurementLine(circuitEvents, sourceComponent, targetWire);
  } else {
    if (movingControl) {
      sourceComponent.controls?.forEach((control) => {
        if (control.qubit === sourceWire) {
          control.qubit = targetWire;
        }
      });
      sourceComponent.controls = sourceComponent.controls?.sort(
        (a, b) => a.qubit - b.qubit,
      );
    } else {
      sourceComponent.targets = [{ qubit: targetWire }];
    }
  }

  // Update parent component targets
  const parentComponent = findParentComponent(
    circuitEvents.componentGrid,
    sourceLocation,
  );
  if (parentComponent) {
    if (parentComponent.type === "Measurement") {
      // Note: this is very confusing with measurements. Maybe the right thing to do
      // will become more apparent if we implement expandable measurements.
      parentComponent.results = getChildTargets(parentComponent);
    } else if (parentComponent.type === "Operation") {
      parentComponent.targets = getChildTargets(parentComponent);
    }
  }
};

/**
 * Add a component into the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceComponent The component to be added.
 * @param targetLocation The location string of the target position.
 * @param targetWire The wire index to add the component to.
 * @param insertNewColumn Whether to insert a new column when adding the component.
 * @returns The added component or null if the addition was unsuccessful.
 */
const addComponent = (
  circuitEvents: CircuitEvents,
  sourceComponent: Component,
  targetLocation: string,
  targetWire: number,
  insertNewColumn: boolean = false,
): Component | null => {
  const targetComponentParent = findParentArray(
    circuitEvents.componentGrid,
    targetLocation,
  );
  const targetLastIndex = locationStringToIndexes(targetLocation).pop();

  if (
    targetComponentParent == null ||
    targetLastIndex == null ||
    sourceComponent == null
  )
    return null;

  const newSourceComponent = _addComp(
    circuitEvents,
    sourceComponent,
    targetComponentParent,
    targetLastIndex,
    targetWire,
    insertNewColumn,
  );
  if (newSourceComponent.type === "Operation") {
    newSourceComponent.targets = [{ qubit: targetWire }];
  }

  return newSourceComponent;
};

/**
 * Remove an component from the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceLocation The location string of the component to be removed.
 */
const removeComponent = (
  circuitEvents: CircuitEvents,
  sourceLocation: string,
) => {
  const sourceComponent = findComponent(
    circuitEvents.componentGrid,
    sourceLocation,
  );
  const sourceComponentParent = findParentArray(
    circuitEvents.componentGrid,
    sourceLocation,
  );

  if (sourceComponent == null || sourceComponentParent == null) return null;

  // Delete sourceComponent
  _removeComp(circuitEvents, sourceComponent, sourceComponentParent);
};

/**
 * Find and remove components in-place based on a predicate function.
 *
 * @param componentGrid The grid of components to search through.
 * @param pred The predicate function to determine which components to remove.
 */
const findAndRemoveComponents = (
  componentGrid: ComponentGrid,
  pred: (comp: Component) => boolean,
) => {
  const inPlaceFilter = (
    grid: ComponentGrid,
    pred: (comp: Component) => boolean,
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

  const recursivePred = (comp: Component) => {
    if (pred(comp)) return true;
    if (comp.children) {
      inPlaceFilter(comp.children, (child) => !recursivePred(child));
    }
    return false;
  };

  inPlaceFilter(componentGrid, (comp) => !recursivePred(comp));
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
 * Add an component to the circuit at the specified location.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceComponent The component to be added.
 * @param targetComponentParent The parent grid where the component will be added.
 * @param targetLastIndex The index within the parent array where the component will be added.
 * @param targetWire The wire index to add the component to.
 * @param insertNewColumn Whether to insert a new column when adding the component.
 * @returns The added component.
 */
const _addComp = (
  circuitEvents: CircuitEvents,
  sourceComponent: Component,
  targetComponentParent: ComponentGrid,
  targetLastIndex: [number, number],
  targetWire: number,
  insertNewColumn: boolean = false,
): Component => {
  const newSourceComponent: Component = JSON.parse(
    JSON.stringify(sourceComponent),
  );
  if (newSourceComponent.type === "Measurement") {
    _addMeasurementLine(circuitEvents, newSourceComponent, targetWire);
  }
  const [colIndex, compIndex] = targetLastIndex;
  if (targetComponentParent[colIndex] == null) {
    targetComponentParent[colIndex] = { components: [] };
  }
  if (insertNewColumn) {
    targetComponentParent.splice(colIndex, 0, {
      components: [newSourceComponent],
    });
  } else {
    targetComponentParent[colIndex].components.splice(
      compIndex,
      0,
      newSourceComponent,
    );
  }
  return newSourceComponent;
};

/**
 * Remove an component from the circuit.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceComponent The component to be removed.
 * @param sourceComponentParent The parent grid from which the component will be removed.
 */
const _removeComp = (
  circuitEvents: CircuitEvents,
  sourceComponent: Component,
  sourceComponentParent: ComponentGrid,
) => {
  if (sourceComponent.dataAttributes === undefined) {
    sourceComponent.dataAttributes = { removed: "true" };
  } else {
    sourceComponent.dataAttributes["removed"] = "true";
  }

  // Find and remove the component in sourceComponentParent
  for (let colIndex = 0; colIndex < sourceComponentParent.length; colIndex++) {
    const col = sourceComponentParent[colIndex];
    const indexToRemove = col.components.findIndex(
      (comp) => comp.dataAttributes && comp.dataAttributes["removed"],
    );
    if (indexToRemove !== -1) {
      col.components.splice(indexToRemove, 1);
      if (col.components.length === 0) {
        sourceComponentParent.splice(colIndex, 1);
      }
      break;
    }
  }

  if (sourceComponent.type === "Measurement") {
    _removeMeasurementLines(circuitEvents, sourceComponent);
  }
};

/**
 * Add a measurement line to the circuit and attach the source measurement.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceMeasurement The measurement to which the measurement line will be added.
 * @param targetQubitWire The wire index to add the measurement line to.
 */
const _addMeasurementLine = (
  circuitEvents: CircuitEvents,
  sourceMeasurement: Measurement,
  targetQubitWire: number,
) => {
  const newNumResults = circuitEvents.qubits[targetQubitWire].numResults
    ? circuitEvents.qubits[targetQubitWire].numResults + 1
    : 1;
  circuitEvents.qubits[targetQubitWire].numResults = newNumResults;
  sourceMeasurement.results = [
    {
      qubit: targetQubitWire,
      result: newNumResults - 1,
    },
  ];
  sourceMeasurement.qubits = [{ qubit: targetQubitWire }];
};

/**
 * Removes all measurement lines of a measure from the circuit and adjust the cIds of the other measurements.
 *
 * @param circuitEvents The CircuitEvents instance to handle circuit-related events.
 * @param sourceMeasurement The measurement gate from which the measurement lines will be removed.
 */
const _removeMeasurementLines = (
  circuitEvents: CircuitEvents,
  sourceMeasurement: Measurement,
) => {
  for (const result of sourceMeasurement.results) {
    const qubit = circuitEvents.qubits[result.qubit];
    if (qubit.numResults != undefined && result.result != undefined) {
      for (const col of circuitEvents.componentGrid) {
        for (const comp of col.components) {
          const controls =
            comp.type === "Measurement" ? comp.qubits : comp.controls;
          if (controls) {
            for (const control of controls) {
              if (
                control.qubit === result.qubit &&
                control.result &&
                control.result > result.result
              ) {
                control.result--;
              }
            }
          }
          const targets =
            comp.type === "Measurement" ? comp.results : comp.targets;
          for (const targetReg of targets) {
            if (
              targetReg.qubit === result.qubit &&
              targetReg.result &&
              targetReg.result > result.result
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
  moveComponent,
  addComponent,
  removeComponent,
  findAndRemoveComponents,
  addControl,
  removeControl,
};
