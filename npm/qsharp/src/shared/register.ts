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
 * Rendering data for qubit register.
 */
export interface RegisterRenderData {
  /** Type of register. */
  type: RegisterType;
  /** y coord of register */
  y: number;
  /** Nested classical registers attached to quantum register. */
  children?: RegisterRenderData[];
}

/**
 * Mapping from qubit IDs to their register render data.
 */
export interface RegisterMap {
  [id: number]: RegisterRenderData;
}
