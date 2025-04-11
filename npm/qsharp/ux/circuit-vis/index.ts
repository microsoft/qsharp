// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Sqore } from "./sqore";
import { CircuitGroup } from "./circuit";

/**
 * Initializes Sqore object with custom styles.
 *
 * @param circuitGroup Group of circuits to be visualized.
 * @param style Custom visualization style.
 */
export const create = (circuitGroup: CircuitGroup): Sqore => {
  return new Sqore(circuitGroup);
};

export { operationListToGrid } from "./utils";

// Export types
export type {
  CircuitGroup,
  Circuit,
  ComponentGrid,
  Column,
  Qubit,
  Operation,
} from "./circuit";
