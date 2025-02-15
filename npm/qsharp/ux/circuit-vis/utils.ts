// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Metadata, GateType } from "./metadata";
import {
  minGateWidth,
  labelPadding,
  labelFontSize,
  argsFontSize,
} from "./constants";
import { Operation } from "./circuit";
import { Register } from "./register";

/**
 * Generate a UUID using `Math.random`.
 * Note: this implementation came from https://stackoverflow.com/questions/105034/how-to-create-guid-uuid
 * and is not cryptographically secure but works for our use case.
 *
 * @returns UUID string.
 */
const createUUID = (): string =>
  "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0,
      v = c == "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });

/**
 * Calculate the width of a gate, given its metadata.
 *
 * @param metadata Metadata of a given gate.
 *
 * @returns Width of given gate (in pixels).
 */
const getGateWidth = ({
  type,
  label,
  displayArgs,
  width,
}: Metadata): number => {
  if (width > 0) return width;

  switch (type) {
    case GateType.Measure:
    case GateType.Cnot:
    case GateType.Swap:
      return minGateWidth;
    default: {
      const labelWidth = _getStringWidth(label);
      const argsWidth =
        displayArgs != null ? _getStringWidth(displayArgs, argsFontSize) : 0;
      const textWidth = Math.max(labelWidth, argsWidth) + labelPadding * 2;
      return Math.max(minGateWidth, textWidth);
    }
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
const _getStringWidth = (
  text: string,
  fontSize: number = labelFontSize,
): number => {
  const canvas: HTMLCanvasElement = document.createElement("canvas");
  const context: CanvasRenderingContext2D | null = canvas.getContext("2d");
  if (context == null) throw new Error("Null canvas");

  context.font = `${fontSize}px Arial`;
  const metrics: TextMetrics = context.measureText(text);
  return metrics.width;
};

/**
 * Find targets of an operation by recursively walking through all of its children controls and targets.
 *
 * Example:
 * Gate Foo contains gate H and gate RX.
 * qIds of Gate H is 1
 * qIds of Gate RX are 1, 2
 * This should return [{qId: 1}, {qId: 2}]
 *
 * @param operation The operation to find targets for.
 * @returns An array of registers with unique qIds.
 */
const getGateTargets = (operation: Operation): Register[] | [] => {
  const _recurse = (operation: Operation) => {
    registers.push(...operation.targets);
    if (operation.controls) {
      registers.push(...operation.controls);
      // If there is more children, keep adding more to registers
      if (operation.children) {
        for (const child of operation.children) {
          _recurse(child);
        }
      }
    }
  };

  const registers: Register[] = [];
  if (operation.children == null) return [];

  // Recursively walkthrough all children to populate registers
  for (const child of operation.children) {
    _recurse(child);
  }

  // Extract qIds from array of object
  // i.e. [{qId: 0}, {qId: 1}, {qId: 1}] -> [0, 1, 1]
  const qIds = registers.map((register) => register.qId);
  const uniqueQIds = Array.from(new Set(qIds));

  // Transform array of numbers into array of qId object
  // i.e. [0, 1] -> [{qId: 0}, {qId: 1}]
  return uniqueQIds.map((qId) => ({
    qId,
    type: 0,
  }));
};

/**
 * Split a location string into an array of indexes.
 *
 * Example:
 * "1-2-3" -> [1, 2, 3]
 *
 * @param location The location string to split.
 * @returns An array of indexes.
 */
const locationStringToIndexes = (location: string): number[] => {
  return location !== ""
    ? location.split("-").map((segment) => parseInt(segment))
    : [];
};

/**
 * Gets the location of an operation, if it has one.
 *
 * @param operation The operation to get the location for.
 * @returns The location string of the operation, or null if it doesn't have one.
 */
const getGateLocationString = (operation: Operation): string | null => {
  if (operation.dataAttributes == null) return null;
  return operation.dataAttributes["location"];
};

export {
  createUUID,
  getGateWidth,
  getGateTargets,
  locationStringToIndexes,
  getGateLocationString,
};
