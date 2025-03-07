// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

/**
 * Type of register.
 */
export enum RegisterType {
  Qubit,
  Classical,
}

/**
 * Represents a register resource.
 */
export interface Register {
  /** Qubit register ID. */
  qubit: number;
  /** Classical register ID. If present, register is classical register. */
  result?: number;
}

/**
 * Metadata for qubit register.
 */
export interface RegisterMetadata {
  /** Type of register. */
  type: RegisterType;
  /** y coord of register */
  y: number;
  /** Nested classical registers attached to quantum register. */
  children?: RegisterMetadata[];
}

/**
 * Mapping from qubit IDs to their register metadata.
 */
export interface RegisterMap {
  [id: number]: RegisterMetadata;
}
