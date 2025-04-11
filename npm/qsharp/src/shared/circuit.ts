// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Register } from "./register.js";

/**
 * Current format version.
 */
export const CURRENT_VERSION = 1;

export interface CircuitGroup {
  circuits: Circuit[];
  version: number;
}

/**
 * Circuit to be visualized.
 */
export interface Circuit {
  /** Array of qubit resources. */
  qubits: Qubit[];
  componentGrid: ComponentGrid;
}

export type ComponentGrid = Column[];

export interface Column {
  components: Component[];
}

/**
 * Represents a component of a circuit. Currently, the only component is an operation.
 * In the future, this may be extended to include other components.
 */
export type Component = Operation;

/**
 * Represents a unique qubit resource bit.
 */
export interface Qubit {
  /** Qubit ID. */
  id: number;
  /** Number of measurement results associated to the qubit. */
  numResults?: number;
}

/**
 * Base type for operations.
 */
export interface BaseOperation {
  /** Gate label. */
  gate: string;
  /** Formatted gate arguments. */
  args?: string[];
  /** The parameters expected for the operation. */
  params?: Parameter[];
  /** Nested operations within this operation. */
  children?: ComponentGrid;

  /** Custom data attributes to attach to gate element.
  Note that this is never written to file, so it is not part of the circuit schema */
  dataAttributes?: DataAttributes;

  /** Whether gate is a conditional operation. */
  isConditional?: boolean;
  /** Specify conditions on when to render operation. */
  conditionalRender?: ConditionalRender;
}

/**
 * Represents a measurement operation and the registers it acts on.
 */
export interface Measurement extends BaseOperation {
  /** Discriminator for the Operation type */
  kind: "measurement";
  /** The qubit registers the gate measures. */
  qubits: Register[];
  /** The classical registers the gate writes to. */
  results: Register[];
}

/**
 * Represents a unitary operation and the registers it acts on.
 */
export interface Unitary extends BaseOperation {
  /** Discriminator for the Operation type */
  kind: "unitary";
  /** Target registers the gate acts on. */
  targets: Register[];
  /** Control registers the gate acts on. */
  controls?: Register[];
  /** Whether gate is an adjoint operation. */
  isAdjoint?: boolean;
}

/**
 * Represents a gate that sets its targets to a specific state.
 */
export interface Ket extends BaseOperation {
  /** Discriminator for the Operation type */
  kind: "ket";
  /** Target registers the gate acts on. */
  targets: Register[];
}

/**
 * Union type for operations.
 */
export type Operation = Unitary | Measurement | Ket;

/**
 * A parameter for an operation.
 */
export interface Parameter {
  /** Parameter name. */
  name: string;
  /** Parameter's Q# type. */
  type: string;
}

/**
 * Conditions on when to render the given operation.
 */
export enum ConditionalRender {
  /** Always rendered. */
  Always,
  /** Render classically-controlled operation when measurement is a zero. */
  OnZero,
  /** Render classically-controlled operation when measurement is a one. */
  OnOne,
  /** Render operation as a group of its nested operations. */
  AsGroup,
}

/**
 * Custom data attributes (e.g. data-{attr}="{val}")
 */
export interface DataAttributes {
  [attr: string]: string;
}
