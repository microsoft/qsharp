/**
 * Helper file with test circuit data for UI tests
 */
import { CircuitGroup } from "../../src/shared/circuit";

/**
 * Creates a simple Bell pair circuit
 */
export function getBellPairCircuit(): CircuitGroup {
  return {
    version: 1,
    circuits: [
      {
        qubits: [{ id: 0 }, { id: 1 }],
        componentGrid: [
          {
            components: [
              { kind: "unitary", gate: "H", targets: [{ qubit: 0 }] },
            ],
          },
          {
            components: [
              {
                kind: "unitary",
                gate: "X",
                targets: [{ qubit: 1 }],
                controls: [{ qubit: 0 }],
              },
            ],
          },
          {
            components: [
              {
                kind: "measurement",
                qubits: [{ qubit: 0 }],
                results: [{ qubit: 0, result: 0 }],
                gate: "M",
              },
              {
                kind: "measurement",
                qubits: [{ qubit: 1 }],
                results: [{ qubit: 1, result: 0 }],
                gate: "M",
              },
            ],
          },
        ],
      },
    ],
  };
}

/**
 * Creates a circuit with parameterized rotation gates
 */
export function getParameterizedCircuit(): CircuitGroup {
  return {
    version: 1,
    circuits: [
      {
        qubits: [{ id: 0 }, { id: 1 }],
        componentGrid: [
          {
            components: [
              {
                kind: "unitary",
                gate: "RX",
                targets: [{ qubit: 0 }],
                args: ["1.5708"],
                params: [{ name: "theta", type: "Double" }],
              },
            ],
          },
          {
            components: [
              {
                kind: "unitary",
                gate: "RY",
                targets: [{ qubit: 1 }],
                args: ["1.5708"],
                params: [{ name: "theta", type: "Double" }],
              },
            ],
          },
        ],
      },
    ],
  };
}

/**
 * Creates a large circuit that would exceed render limits
 */
export function getLargeCircuit(): CircuitGroup {
  return {
    version: 1,
    circuits: [
      {
        qubits: Array.from({ length: 1001 }, (_, i) => ({ id: i })),
        componentGrid: [],
      },
    ],
  };
}

/**
 * Creates a multi-circuit group that contains multiple circuits
 */
export function getMultiCircuitGroup(): CircuitGroup {
  return {
    version: 1,
    circuits: [
      // First circuit
      {
        qubits: [{ id: 0 }, { id: 1 }],
        componentGrid: [
          {
            components: [
              { kind: "unitary", gate: "H", targets: [{ qubit: 0 }] },
            ],
          },
        ],
      },
      // Second circuit
      {
        qubits: [{ id: 0 }, { id: 1 }, { id: 2 }],
        componentGrid: [
          {
            components: [
              { kind: "unitary", gate: "X", targets: [{ qubit: 0 }] },
            ],
          },
        ],
      },
    ],
  };
}
