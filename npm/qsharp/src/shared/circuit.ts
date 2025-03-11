// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Register } from "./register.js";

/**
 * Current format version.
 */
export const CURRENT_VERSION = 1;

export interface CircuitGroup {
  circuits: Circuit[];
  version?: number;
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

export type Component = Operation;

/**
 * Update circuit group to current format version.
 */
export const updateToCurrentVersion = (
  circuits: CircuitGroup,
): CircuitGroup => {
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
  /** Qubit ID. */
  id: number;
  /** Number of measurement results associated to the qubit. */
  numResults?: number;
}

/**
 * Represents an operation and the registers it acts on.
 */
export interface Operation {
  /** Gate label. */
  gate: string;
  /** Formatted gate arguments. */
  args?: string[];
  /** Nested operations within this operation. */
  children?: ComponentGrid;
  /** Whether gate is a measurement operation. */
  isMeasurement?: boolean;
  /** Whether gate is a conditional operation. */
  isConditional?: boolean;
  /** Whether gate is an adjoint operation. */
  isAdjoint?: boolean;
  /** Control registers the gate acts on. */
  controls?: Register[];
  /** Target registers the gate acts on. */
  targets: Register[];
  /** Specify conditions on when to render operation. */
  conditionalRender?: ConditionalRender;
  /** Custom data attributes to attach to gate element.
  Note that this is never written to file, so it is not part of the circuit schema */
  dataAttributes?: DataAttributes;
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
