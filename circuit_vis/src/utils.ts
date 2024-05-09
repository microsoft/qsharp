// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Metadata, GateType } from './metadata';
import { minGateWidth, labelPadding, labelFontSize, argsFontSize } from './constants';

/**
 * Generate a UUID using `Math.random`.
 * Note: this implementation came from https://stackoverflow.com/questions/105034/how-to-create-guid-uuid
 * and is not cryptographically secure but works for our use case.
 *
 * @returns UUID string.
 */
const createUUID = (): string =>
    'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
        const r = (Math.random() * 16) | 0,
            v = c == 'x' ? r : (r & 0x3) | 0x8;
        return v.toString(16);
    });

/**
 * Calculate the width of a gate, given its metadata.
 *
 * @param metadata Metadata of a given gate.
 *
 * @returns Width of given gate (in pixels).
 */
const getGateWidth = ({ type, label, displayArgs, width }: Metadata): number => {
    if (width > 0) return width;

    switch (type) {
        case GateType.Measure:
        case GateType.Cnot:
        case GateType.Swap:
            return minGateWidth;
        default:
            const labelWidth = _getStringWidth(label);
            const argsWidth = displayArgs != null ? _getStringWidth(displayArgs, argsFontSize) : 0;
            const textWidth = Math.max(labelWidth, argsWidth) + labelPadding * 2;
            return Math.max(minGateWidth, textWidth);
    }
};

/**
 * Get the width of a string with font-size `fontSize` and font-family Arial.
 *
 * @param text     Input string.
 * @param fontSize Font size of `text`.
 *
 * @returns Pixel width of given string.
 */
const _getStringWidth = (text: string, fontSize: number = labelFontSize): number => {
    const canvas: HTMLCanvasElement = document.createElement('canvas');
    const context: CanvasRenderingContext2D | null = canvas.getContext('2d');
    if (context == null) throw new Error('Null canvas');

    context.font = `${fontSize}px Arial`;
    const metrics: TextMetrics = context.measureText(text);
    return metrics.width;
};

export { createUUID, getGateWidth, _getStringWidth };
