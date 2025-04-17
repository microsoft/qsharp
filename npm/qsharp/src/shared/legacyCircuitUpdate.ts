// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { getMinMaxRegIdx } from "../utils.js";
import {
  Circuit,
  CircuitGroup,
  ComponentGrid,
  CURRENT_VERSION,
  Operation,
  Qubit,
} from "./circuit.js";

/**
 * Ensures that the given circuit object is a CircuitGroup, doing any
 * necessary conversions from Circuit or legacy formats.
 *
 * @param circuit The circuit to convert.
 * @returns The converted CircuitGroup.
 */
export function toCircuitGroup(circuit: any): CircuitGroup {
  const emptyCircuit: Circuit = {
    qubits: [],
    componentGrid: [],
  };

  const emptyCircuitGroup: CircuitGroup = {
    version: CURRENT_VERSION,
    circuits: [emptyCircuit],
  };

  if (circuit && Object.keys(circuit).length === 0) {
    return emptyCircuitGroup;
  }

  if (circuit?.version) {
    const version = circuit.version;
    // If it has a "version" field, it is up-to-date
    if (isCircuitGroup(circuit)) {
      // If it's already a CircuitGroup, return it as is
      return circuit;
    } else if (isCircuit(circuit)) {
      // If it's a Circuit, wrap it in a CircuitGroup
      return {
        version,
        circuits: [circuit],
      };
    } else {
      console.error(
        "Unknown schema: circuit is neither a CircuitGroup nor a Circuit.",
      );
      return emptyCircuitGroup;
    }
  } else if (isCircuit(circuit)) {
    // If it's a Circuit without a version, wrap it in a CircuitGroup
    return {
      version: CURRENT_VERSION,
      circuits: [circuit],
    };
  } else if (circuit?.operations) {
    // Legacy schema: convert to CircuitGroup
    if (circuit.qubits === undefined || !Array.isArray(circuit.qubits)) {
      console.error("Unknown schema: circuit is missing qubit information.");
      return emptyCircuitGroup;
    }

    const qubits: Qubit[] = circuit.qubits.map((qubit: any) => {
      return {
        id: qubit.id,
        numResults: qubit.numChildren || 0, // Rename "numChildren" to "numResults"
      };
    });

    const componentGrid = operationListToGrid(
      circuit.operations.map(toOperation),
      qubits.length,
    );

    return {
      version: CURRENT_VERSION,
      circuits: [
        {
          qubits,
          componentGrid,
        },
      ],
    };
  } else {
    console.error("Unknown schema: circuit does not match any known format.");
    return emptyCircuitGroup;
  }
}

/**
 * Converts a legacy operation object to the new Operation format.
 *
 * @param op The operation to convert.
 * @returns The converted Operation.
 */
function toOperation(op: any): Operation {
  let targets = [];
  if (op.targets) {
    targets = op.targets.map((t: any) => {
      return {
        qubit: t.qId,
        result: t.cId,
      };
    });
  }
  let controls = undefined;
  if (op.controls) {
    controls = op.controls.map((c: any) => {
      return {
        qubit: c.qId,
        result: c.cId,
      };
    });
  }

  if (op.isMeasurement) {
    return {
      ...op,
      kind: "measurement",
      qubits: controls || [],
      results: targets,
    } as Operation;
  } else {
    const ket = getKetLabel(op.gate);
    if (ket.length > 0) {
      return {
        ...op,
        kind: "ket",
        gate: ket,
        targets,
      };
    } else {
      const convertedOp: Operation = {
        ...op,
        kind: "unitary",
        targets,
        controls,
      };
      if (op.displayArgs) {
        convertedOp.args = [op.displayArgs];
        // Assume the parameter is always "theta" for now
        convertedOp.params = [{ name: "theta", type: "Double" }];
      }
      if (op.children) {
        convertedOp.children = [
          {
            components: op.children.map((child: any) => toOperation(child)),
          },
        ];
      }
      return convertedOp;
    }
  }
}

/**
 * Checks if the given object is a CircuitGroup.
 *
 * @param circuit The object to check.
 * @returns True if the object is a CircuitGroup, false otherwise.
 */
function isCircuitGroup(circuit: any): circuit is CircuitGroup {
  return circuit && Array.isArray(circuit.circuits);
}

/**
 * Checks if the given object is a Circuit.
 *
 * @param circuit The object to check.
 * @returns True if the object is a Circuit, false otherwise.
 */
function isCircuit(circuit: any): circuit is Circuit {
  return (
    circuit &&
    Array.isArray(circuit.qubits) &&
    Array.isArray(circuit.componentGrid)
  );
}

/**
 * Get the label from a ket string.
 *
 * @param ket The ket string to extract the label from.
 * @returns The label extracted from the ket string.
 */
function getKetLabel(ket: string): string {
  // Check that the ket conforms to the format |{label}> or |{label}⟩
  const ketRegex = /^\|([^\s〉⟩〉>]+)(?:[〉⟩〉>])$/;

  // Match the ket string against the regex
  const match = ket.match(ketRegex);

  // If valid, return the inner label (captured group 1), otherwise return an empty string
  return match ? match[1] : "";
}

/**
 * Converts a list of operations into a 2D grid of operations in col-row format.
 * Operations will be left-justified as much as possible in the resulting grid.
 * Children operations are recursively converted into a grid.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns A 2D array of operations.
 */
function operationListToGrid(
  operations: Operation[],
  numQubits: number,
): ComponentGrid {
  operations.forEach((op) => {
    // The children data structure is a grid, so checking if it is
    // length 1 is actually checking if it has a single column,
    // or in other words, we are checking if its children are in a single list.
    // If the operation has children in a single list, it needs to be converted to a grid.
    // If it was already converted to a grid, but the grid was still a single list,
    // then doing it again won't effect anything.
    if (op.children && op.children.length == 1) {
      op.children = operationListToGrid(op.children[0].components, numQubits);
    }
  });

  return removePadding(operationListToPaddedArray(operations, numQubits)).map(
    (col) => ({
      components: col,
    }),
  );
}

/**
 * Converts a list of operations into a padded 2D array of operations.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns A 2D array of operations padded with `null`s.
 */
function operationListToPaddedArray(
  operations: Operation[],
  numQubits: number,
): (Operation | null)[][] {
  if (operations.length === 0) return [];

  // Group operations based on registers
  const groupedOps: number[][] = groupOperations(operations, numQubits);

  // Align operations on multiple registers
  const alignedOps: (number | null)[][] = transformToColRow(
    alignOps(groupedOps),
  );

  const operationArray: (Operation | null)[][] = alignedOps.map((col) =>
    col.map((opIdx) => {
      if (opIdx == null) return null;
      return operations[opIdx];
    }),
  );

  return operationArray;
}

/**
 * Removes padding (`null` values) from a 2D array of operations.
 *
 * @param operations 2D array of operations padded with `null`s.
 *
 * @returns A 2D array of operations without `null` values.
 */
function removePadding(operations: (Operation | null)[][]): Operation[][] {
  return operations.map((col) => col.filter((op) => op != null));
}

/**
 * Transforms a row-col 2D array into an equivalent col-row 2D array.
 *
 * @param alignedOps 2D array of operations in row-col format.
 *
 * @returns 2D array of operations in col-row format.
 */
function transformToColRow(
  alignedOps: (number | null)[][],
): (number | null)[][] {
  if (alignedOps.length === 0) return [];

  const numRows = alignedOps.length;
  const numCols = Math.max(...alignedOps.map((row) => row.length));

  const colRowArray: (number | null)[][] = Array.from({ length: numCols }, () =>
    Array(numRows).fill(null),
  );

  for (let row = 0; row < numRows; row++) {
    for (let col = 0; col < alignedOps[row].length; col++) {
      colRowArray[col][row] = alignedOps[row][col];
    }
  }

  return colRowArray;
}

/**
 * Group gates provided by operations into their respective registers.
 *
 * @param operations Array of operations.
 * @param numQubits  Number of qubits in the circuit.
 *
 * @returns 2D array of indices where `groupedOps[i][j]` is the index of the operations
 *          at register `i` and column `j` (not yet aligned/padded).
 */
function groupOperations(
  operations: Operation[],
  numQubits: number,
): number[][] {
  const groupedOps: number[][] = Array.from(
    Array(numQubits),
    () => new Array(0),
  );
  operations.forEach((operation, instrIdx) => {
    const [minRegIdx, maxRegIdx] = getMinMaxRegIdx(operation, numQubits);
    // Add operation also to registers that are in-between target registers
    // so that other gates won't render in the middle.
    for (let i = minRegIdx; i <= maxRegIdx; i++) {
      groupedOps[i].push(instrIdx);
    }
  });
  return groupedOps;
}

/**
 * Aligns operations by padding registers with `null`s to make sure that multiqubit
 * gates are in the same column.
 * e.g. ---[x]---[x]--
 *      ----------|---
 *
 * @param ops 2D array of operations. Each row represents a register
 *            and the operations acting on it (in-order).
 *
 * @returns 2D array of aligned operations padded with `null`s.
 */
function alignOps(ops: number[][]): (number | null)[][] {
  let maxNumOps: number = Math.max(0, ...ops.map((regOps) => regOps.length));
  let col = 0;
  // Deep copy ops to be returned as paddedOps
  const paddedOps: (number | null)[][] = ops.map((regOps) => [...regOps]);
  while (col < maxNumOps) {
    for (let regIdx = 0; regIdx < paddedOps.length; regIdx++) {
      const reg: (number | null)[] = paddedOps[regIdx];
      if (reg.length <= col) continue;

      // Should never be null (nulls are only padded to previous columns)
      const opIdx: number | null = reg[col];

      // Get position of gate
      const targetsPos: number[] = paddedOps.map((regOps) =>
        regOps.indexOf(opIdx),
      );
      const gatePos: number = Math.max(-1, ...targetsPos);

      // If current column is not desired gate position, pad with null
      if (col < gatePos) {
        paddedOps[regIdx].splice(col, 0, null);
        maxNumOps = Math.max(maxNumOps, paddedOps[regIdx].length);
      }
    }
    col++;
  }
  return paddedOps;
}
