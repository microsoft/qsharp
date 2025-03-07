// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Sqore } from "./sqore";
import { StyleConfig } from "./styles";
import { CircuitGroup } from "./circuit";

/**
 * Initializes Sqore object with custom styles.
 *
 * @param circuitGroup Group of circuits to be visualized.
 * @param style Custom visualization style.
 */
export const create = (
  circuitGroup: CircuitGroup,
  style: StyleConfig | string = {},
): Sqore => {
  return new Sqore(circuitGroup, style);
};

export { STYLES } from "./styles";

// Export types
export type { StyleConfig } from "./styles";
export type { CircuitGroup, Circuit, Qubit, Operation } from "./circuit";
