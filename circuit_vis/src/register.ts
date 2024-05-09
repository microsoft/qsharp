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
    /** Type of register. If missing defaults to Qubit. */
    type?: RegisterType;
    /** Qubit register ID. */
    qId: number;
    /** Classical register ID (if classical register). */
    cId?: number;
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
