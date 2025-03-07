// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Register } from "./register.js";

/**
 * Current format version.
 */
export const CURRENT_VERSION = "1.0.0";

export interface CircuitFile {
  version: string;
  circuits: Circuit[];
  name: string;
}

/**
 * Circuit to be visualized.
 */
export interface Circuit {
  /** Array of qubit resources. */
  qubits: Qubit[];
  components: Component[][];
}

/**
 * Update circuit file to current format version.
 */
export const updateToCurrentVersion = (circuits: CircuitFile): CircuitFile => {
  if (circuits.version === CURRENT_VERSION) {
    return circuits;
  }
  return {
    ...circuits,
    version: CURRENT_VERSION,
  };
};

/**
 * Represents a unique qubit resource bit.
 */
export interface Qubit {
  name?: string;
  measurements?: { name?: string }[];
}

//type Component = Operation | ControlFlow;
type Component = Operation;

/**
 * Represents an operation and the registers it acts on.
 */
export interface Operation {
  /** Gate label. */
  gate: string;
  /** Formatted gate arguments to be displayed. */
  displayArgs?: string[];
  /** Number of columns to span. */
  columnSpan?: number;
  /** Whether gate is a measurement operation. */
  isMeasurement?: boolean;
  /** Whether gate is an adjoint operation. */
  isAdjoint?: boolean;
  /** Control registers the gate acts on. */
  controls?: Register[];
  /** Target registers the gate acts on. */
  targets: Register[];
  /** Custom data attributes to attach to gate element. */
  dataAttributes?: DataAttributes;
}

/**
 * Custom data attributes (e.g. data-{attr}="{val}")
 */
export interface DataAttributes {
  [attr: string]: string;
}

export interface ControlFlow {
  type: "While" | "If" | "ElseIf" | "Else" | "For" | "Custom";
  columnSpan?: number;

  // If empty, control flow box covers entire column.
  // Not sure if we want this, or if we always want to cover the entire column.
  // We may not want to have the ability to have discontinuous targets for control-flows, and instead just have a `wireSpan` instead.
  targets?: Register[];

  /** This is the Q# condition for the While, If, ElseIf cases, and a Q# range for the For case. It can be any valid Q# expression for the Custom case. */
  expression?: string;

  /** This is for the ElseIf and Else cases of an If case, to relate them. */
  // Is there a better way to do this?
  relatedFlows?: ControlFlow[];

  /** Nested components within this control-flow. */
  operations: Component[][];
}

/**
 * Represents a register resource.
 */
// export interface Register {
//   /** Qubit register ID. */
//   qId: number;
//   /** Classical register ID (if classical register). */
//   cId?: number;
// }
