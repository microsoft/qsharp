// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Sqore } from './sqore';
import { Circuit } from './circuit';
import { StyleConfig } from './styles';

/**
 * Render `circuit` into `container` at the specified layer depth.
 *
 * @param circuit Circuit to be visualized.
 * @param container HTML element for rendering visualization into.
 * @param style Custom visualization style.
 * @param renderDepth Initial layer depth at which to render gates.
 */
export const draw = (
    circuit: Circuit,
    container: HTMLElement,
    style: StyleConfig | string = {},
    renderDepth = 0,
): void => {
    const sqore = new Sqore(circuit, style);
    sqore.draw(container, renderDepth);
};

export { STYLES } from './styles';

// Export types
export type { StyleConfig } from './styles';
export type { Circuit, Qubit, Operation } from './circuit';
