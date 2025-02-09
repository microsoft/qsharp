// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Sqore } from "./sqore";
import { Circuit } from "./circuit";
import { StyleConfig } from "./styles";

/**
 * Render `circuit` into `container` at the specified layer depth.
 *
 * @param circuit Circuit to be visualized.
 * @param style Custom visualization style.
 */
export const create = (
  circuit: Circuit,
  style: StyleConfig | string = {},
): Sqore => {
  return new Sqore(circuit, style);
};

export { STYLES } from "./styles";

// Export types
export type { StyleConfig } from "./styles";
export type { Circuit, Qubit, Operation } from "./circuit";
