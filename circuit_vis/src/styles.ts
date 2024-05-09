// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

/**
 * Provides configuration for CSS styles of visualization.
 */
export interface StyleConfig {
    /** Line stroke style. */
    lineStroke?: string;
    /** Line width. */
    lineWidth?: number;
    /** Text colour. */
    textColour?: string;
    /** Single qubit unitary fill colour. */
    unitary?: string;
    /** Oplus circle fill colour. */
    oplus?: string;
    /** Measurement gate fill colour. */
    measure?: string;
    /** Measurement unknown primary colour. */
    classicalUnknown?: string;
    /** Measurement zero primary colour. */
    classicalZero?: string;
    /** Measurement one primary colour. */
    classicalOne?: string;
    /** Measurement zero text colour */
    classicalZeroText?: string;
    /** Measurement one text colour */
    classicalOneText?: string;
}

const defaultStyle: StyleConfig = {
    lineStroke: '#000000',
    lineWidth: 1,
    textColour: '#000000',
    unitary: '#D9F1FA',
    oplus: '#FFFFFF',
    measure: '#FFDE86',
    classicalUnknown: '#E5E5E5',
    classicalZero: '#C40000',
    classicalOne: '#4059BD',
    classicalZeroText: '#FFFFFF',
    classicalOneText: '#FFFFFF',
};

const blackAndWhiteStyle: StyleConfig = {
    lineStroke: '#000000',
    lineWidth: 1,
    textColour: '#000000',
    unitary: '#FFFFFF',
    oplus: '#FFFFFF',
    measure: '#FFFFFF',
    classicalUnknown: '#FFFFFF',
    classicalZero: '#000000',
    classicalOne: '#000000',
    classicalZeroText: '#FFFFFF',
    classicalOneText: '#FFFFFF',
};

const invertedStyle: StyleConfig = {
    lineStroke: '#FFFFFF',
    lineWidth: 1,
    textColour: '#FFFFFF',
    unitary: '#000000',
    oplus: '#000000',
    measure: '#000000',
    classicalUnknown: '#000000',
    classicalZero: '#FFFFFF',
    classicalOne: '#FFFFFF',
    classicalZeroText: '#000000',
    classicalOneText: '#000000',
};

/**
 * Set of default styles.
 */
export const STYLES: { [name: string]: StyleConfig } = {
    /** Default style with coloured gates. */
    Default: defaultStyle,
    /** Black and white style. */
    BlackAndWhite: blackAndWhiteStyle,
    /** Inverted black and white style (for black backgrounds). */
    Inverted: invertedStyle,
};

/**
 * CSS style script to be injected into visualization SVG.
 *
 * @param customStyle Custom style configuration.
 *
 * @returns String containing CSS style script.
 */
export const style = (customStyle: StyleConfig = {}): string => {
    const styleConfig = { ...defaultStyle, ...customStyle };

    return `${_defaultGates(styleConfig)}
    ${_classicallyControlledGates(styleConfig)}
    ${_expandCollapse}`;
};

const _defaultGates = (styleConfig: StyleConfig): string => `
    line,
    circle,
    rect {
        stroke: ${styleConfig.lineStroke};
        stroke-width: ${styleConfig.lineWidth};
    }
    text {
        fill: ${styleConfig.textColour};
        dominant-baseline: middle;
        text-anchor: middle;
        font-family: Arial;
    }
    .control-dot {
        fill: ${styleConfig.lineStroke};
    }
    .oplus line, .oplus circle {
        fill: ${styleConfig.oplus};
        stroke-width: 2;
    }
    .gate-unitary {
        fill: ${styleConfig.unitary};
    }
    .gate-measure {
        fill: ${styleConfig.measure};
    }
    rect.gate-swap {
        fill: transparent;
        stroke: transparent;
    }
    .arc-measure {
        stroke: ${styleConfig.lineStroke};
        fill: none;
        stroke-width: ${styleConfig.lineWidth};
    }
    .register-classical {
        stroke-width: ${(styleConfig.lineWidth || 0) / 2};
    }`;

const _classicallyControlledGates = (styleConfig: StyleConfig): string => {
    const gateOutline = `
    .classically-controlled-one .classical-container,
    .classically-controlled-one .classical-line {
        stroke: ${styleConfig.classicalOne};
        stroke-width: ${(styleConfig.lineWidth || 0) + 0.3};
        fill: ${styleConfig.classicalOne};
        fill-opacity: 0.1;
    }
    .classically-controlled-zero .classical-container,
    .classically-controlled-zero .classical-line {
        stroke: ${styleConfig.classicalZero};
        stroke-width: ${(styleConfig.lineWidth || 0) + 0.3};
        fill: ${styleConfig.classicalZero};
        fill-opacity: 0.1;
    }`;
    const controlBtn = `
    .classically-controlled-btn {
        cursor: pointer;
    }
    .classically-controlled-unknown .classically-controlled-btn {
        fill: ${styleConfig.classicalUnknown};
    }
    .classically-controlled-one .classically-controlled-btn {
        fill: ${styleConfig.classicalOne};
    }
    .classically-controlled-zero .classically-controlled-btn {
        fill: ${styleConfig.classicalZero};
    }`;

    const controlBtnText = `
    .classically-controlled-btn text {
        dominant-baseline: middle;
        text-anchor: middle;
        stroke: none;
        font-family: Arial;
    }
    .classically-controlled-unknown .classically-controlled-btn text {
        fill: ${styleConfig.textColour};
    }
    .classically-controlled-one .classically-controlled-btn text {
        fill: ${styleConfig.classicalOneText};
    }
    .classically-controlled-zero .classically-controlled-btn text {
        fill: ${styleConfig.classicalZeroText};
    }`;

    return `
    .hidden {
        display: none;
    }
    .classically-controlled-unknown {
        opacity: 0.25;
    }

    ${gateOutline}
    ${controlBtn}
    ${controlBtnText}`;
};

const _expandCollapse = `
    .qviz .gate-collapse,
    .qviz .gate-expand {
        opacity: 0;
        transition: opacity 1s;
    }

    .qviz:hover .gate-collapse,
    .qviz:hover .gate-expand {
        visibility: visible;
        opacity: 0.2;
        transition: visibility 1s;
        transition: opacity 1s;
    }

    .gate-expand, .gate-collapse {
        cursor: pointer;
    }

    .gate-collapse circle,
    .gate-expand circle {
        fill: white;
        stroke-width: 2px;
        stroke: black;
    }
    .gate-collapse path,
    .gate-expand path {
        stroke-width: 4px;
        stroke: black;
    }

    .gate:hover > .gate-collapse,
    .gate:hover > .gate-expand {
        visibility: visible;
        opacity: 1;
        transition: opacity 1s;
    }`;
