import { CURRENT_VERSION } from "../../npm/qsharp/dist/shared/circuit";

export const sampleCircuit = {
  operations: [
    [
      {
        gate: "H",
        targets: [
          {
            qId: 0,
            type: 0,
          },
        ],
      },
    ],
    [
      {
        gate: "X",
        isControlled: true,
        controls: [
          {
            qId: 0,
            type: 0,
          },
        ],
        targets: [
          {
            qId: 1,
            type: 0,
          },
        ],
      },
    ],
    [
      {
        gate: "Measure",
        isMeasurement: true,
        controls: [
          {
            qId: 0,
            type: 0,
          },
        ],
        targets: [
          {
            qId: 0,
            type: 1,
            cId: 0,
          },
        ],
      },
      {
        gate: "Measure",
        isMeasurement: true,
        controls: [
          {
            qId: 1,
            type: 0,
          },
        ],
        targets: [
          {
            qId: 1,
            type: 1,
            cId: 0,
          },
        ],
      },
    ],
  ],
  qubits: [
    {
      id: 0,
      numChildren: 1,
    },
    {
      id: 1,
      numChildren: 1,
    },
  ],
  version: CURRENT_VERSION,
};
