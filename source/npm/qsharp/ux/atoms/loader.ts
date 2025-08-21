// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const machineLayout = {
  cols: 16,
  readoutRows: 2,
  interactionRows: 1,
  storageRows: 16,
};

export type MachineLayout = typeof machineLayout;

export type TraceData = {
  metadata: any;
  qubits: Array<[number, number]>;
  steps: Array<{
    id: string | number;
    ops: Array<string>;
  }>;
};

export function getMachine(): MachineLayout {
  return machineLayout;
}

export function getSampleTrace(): TraceData {
  return {
    metadata: { name: "test-rosebud-1a" },
    qubits: [
      [3, 0],
      [3, 1],
      [3, 2],
      [3, 3],
      [3, 4],
      [3, 5],
      [4, 0],
      [4, 1],
      [4, 2],
      [4, 3],
      [4, 4],
      [4, 5],
      [5, 0],
      [5, 1],
      [5, 2],
      [5, 3],
      [5, 4],
      [5, 5],
      [6, 0],
      [6, 1],
      [6, 2],
      [6, 3],
      [6, 4],
      [6, 5],
    ],
    steps: [
      { id: 1, ops: ["sx 0", "sx 1", "sx 2", "sx 3"] },
      {
        id: 2,
        ops: ["move(2, 0) 0", "move(2, 1) 1", "move(2, 2) 2", "move(2, 3) 3"],
      },
      { id: 3, ops: ["cz 0, 1", "cz 2, 3"] },
      {
        id: 4,
        ops: ["move(3, 0) 0", "move(3, 1) 1", "move(3, 2) 2", "move(3, 3) 3"],
      },
      { id: 5, ops: ["rz(0.7854) 0", "rz(0.7854) 2"] },
      { id: 6, ops: ["move(1, 0) 1", "move(1, 1) 3"] },
      { id: 7, ops: ["mz 1", "mz 3"] },
      { id: 8, ops: ["reset 1", "reset 3"] },
      { id: 9, ops: ["move(3, 1) 1", "move(3, 3) 3"] },
      { id: 9, ops: ["rz(0.7854) 8", "rz(0.7854) 10"] },
      { id: 10, ops: ["sx 8", "sx 9", "sx 10", "sx 11"] },
      {
        id: 11,
        ops: ["move(2, 2) 8", "move(2, 3) 9", "move(2, 4) 10", "move(2, 5) 11"],
      },
      { id: 12, ops: ["cz 8, 9", "cz 10, 11"] },
      {
        id: 13,
        ops: ["move(4, 2) 8", "move(4, 3) 9", "move(4, 4) 10", "move(4, 5) 11"],
      },
      { id: 14, ops: ["move(1, 3) 9", "move(1, 5) 11"] },
      { id: 15, ops: ["mz 9", "mz 11"] },
      { id: 16, ops: ["reset 9", "reset 11"] },
      { id: 17, ops: ["move(4, 3) 9", "move(4, 5) 11"] },
    ],
  };
}
