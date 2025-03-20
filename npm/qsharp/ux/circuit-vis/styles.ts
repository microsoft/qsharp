// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { gateHeight, minGateWidth, minToolboxHeight } from "./constants";
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
  lineStroke: "#000000",
  lineWidth: 1,
  textColour: "#000000",
  unitary: "#D9F1FA",
  oplus: "#FFFFFF",
  measure: "#FFDE86",
  classicalUnknown: "#E5E5E5",
  classicalZero: "#C40000",
  classicalOne: "#4059BD",
  classicalZeroText: "#FFFFFF",
  classicalOneText: "#FFFFFF",
};

const blackAndWhiteStyle: StyleConfig = {
  lineStroke: "#000000",
  lineWidth: 1,
  textColour: "#000000",
  unitary: "#FFFFFF",
  oplus: "#FFFFFF",
  measure: "#FFFFFF",
  classicalUnknown: "#FFFFFF",
  classicalZero: "#000000",
  classicalOne: "#000000",
  classicalZeroText: "#FFFFFF",
  classicalOneText: "#FFFFFF",
};

const invertedStyle: StyleConfig = {
  lineStroke: "#FFFFFF",
  lineWidth: 1,
  textColour: "#FFFFFF",
  unitary: "#000000",
  oplus: "#000000",
  measure: "#000000",
  classicalUnknown: "#000000",
  classicalZero: "#FFFFFF",
  classicalOne: "#FFFFFF",
  classicalZeroText: "#000000",
  classicalOneText: "#000000",
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
    ${_expandCollapse}
    ${_draggable}
    ${_panel}
    ${_contextMenu}
    ${_customPromptOverlay}`;
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
    }
`;

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
    }
`;

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
    }
`;

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
    }
`;

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
    }
`;

const _draggable = `
    text {
        user-select: none;
        pointer-events: none;
    }
    .dropzone-layer {
        display: none;
    }
    .dropzone {
        fill-opacity: 0%;
        stroke-opacity: 0%;
    }
    .dropzone:hover {
        fill: var(--vscode-editor-selectionBackground, #EC7063);
        fill-opacity: 50%;
    }
    .dropzone-full-wire {
        fill-opacity: 0%;
        stroke-opacity: 0%;
    }
    .dropzone-full-wire:hover {
        fill: var(--vscode-editor-selectionBackground, #EC7063);
        fill-opacity: 50%;
    }
    .grab {
        cursor: grab;
    }
    .moving {
        cursor: move;
    }
    .copying {
        cursor: copy;
    }
`;

const _panel = `
    .container {
        display: flex;
    }
    .edit-panel {
        display: flex;
        flex-direction: column;
    }
    .title {
        margin: 0 0 4px;
        font-size: 1.2rem;
        user-select: none;
    }
    .block {
        display: block;
    }
    .target-input, .controls-input, .parameters-input {
        margin-bottom: 8px;
    }
    .toolbox-panel {
        width: -webkit-fill-available;
        min-height: ${minToolboxHeight}px;
    }
    .toolbox-panel-svg {
        width: -webkit-fill-available;
        min-height: ${minToolboxHeight}px;
    }
    .ghost {
        width: ${minGateWidth}px;
        height: ${gateHeight}px;
        position: absolute;
        z-index: 1;
        pointer-events: none;
    }
    .qubit-line-control {
        display: flex;
        justify-content: flex-start;
        align-items: center;
        margin-bottom: 8px;
    }
    .add-qubit-line, .remove-qubit-line {
        border-radius: 4px;
        padding: 8px 16px;
        margin: 4px;
        cursor: pointer;
        font-size: 14px;
        user-select: none;
        background: var(--vscode-button-background, #007acc);
        color: var(--vscode-button-foreground, #ffffff);
        border: 1px solid var(--vscode-button-border, #005a9e);
    }
    .add-qubit-line:hover, .remove-qubit-line:hover {
        background: var(--vscode-button-hoverBackground, #005a9e);
    }
    .add-qubit-line:disabled, .remove-qubit-line:disabled {
        background: var(--vscode-button-secondaryBackground, #d4d4d4);
        color: var(--vscode-disabledForeground, #a1a1a1);
        border: 1px solid var(--vscode-button-secondaryBackground, #d4d4d4);
        cursor: not-allowed;
    }
`;

const _contextMenu = `
    .context-menu {
        position: absolute;
        background-color: var(--vscode-menu-background, #ffffff);
        border: 1px solid var(--vscode-menu-border, #cccccc);
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
        z-index: 1000;
        padding: 5px 0;
        border-radius: 4px;
    }

    .context-menu-option {
        padding: 8px 12px;
        cursor: pointer;
        white-space: nowrap;
        color: var(--vscode-menu-foreground, #000000);
    }

    .context-menu-option:hover {
        background-color: var(--vscode-menu-selectionBackground, #f0f0f0);
        color: var(--vscode-menu-selectionForeground, #000000);
    }
`;

const _customPromptOverlay = `
    .custom-prompt-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .custom-prompt-container {
        background: var(--vscode-editor-background, #ffffff);
        color: var(--vscode-editor-foreground, #000000);
        padding: 20px;
        border-radius: 5px;
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
        text-align: center;
    }

    .custom-prompt-message {
        margin-bottom: 10px;
        color: var(--vscode-editor-foreground, #000000);
    }

    .custom-prompt-input {
        width: 100%;
        padding: 5px;
        margin-bottom: 10px;
        background: var(--vscode-input-background, #ffffff);
        color: var(--vscode-input-foreground, #000000);
        border: 1px solid var(--vscode-input-border, #cccccc);
    }

    .custom-prompt-buttons {
        display: flex;
        justify-content: space-between;
    }

    .custom-prompt-button {
        padding: 5px 10px;
        margin: 0 5px;
        background: var(--vscode-button-background, #007acc);
        color: var(--vscode-button-foreground, #ffffff);
        border: 1px solid var(--vscode-button-border, #005a9e);
        cursor: pointer;
    }

    .custom-prompt-button:hover {
        background: var(--vscode-button-hoverBackground, #005a9e);
    }

    .custom-prompt-button:disabled {
        background: var(--vscode-button-secondaryBackground, #d4d4d4); /* Disabled background */
        color: var(--vscode-disabledForeground, #a1a1a1); /* Disabled text color */
        border: 1px solid var(--vscode-button-secondaryBackground, #d4d4d4); /* Disabled border */
        cursor: not-allowed; /* Show not-allowed cursor */
    }

    .pi-button {
        width: 40px;
        height: 40px;
        padding: 0;
        text-align: center;
        line-height: 40px;
        font-size: 16px;
    }
`;
