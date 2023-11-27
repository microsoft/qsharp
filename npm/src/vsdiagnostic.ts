// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type VSDiagnostic } from "../lib/web/qsc_wasm.js";
export { type VSDiagnostic } from "../lib/web/qsc_wasm.js";
import { log } from "./log.js";

// The QSharp compiler returns positions in utf-8 code unit positions (basically a byte[]
// index), however VS Code and Monaco handle positions as utf-16 code unit positions
// (basically JavaScript string index positions). Thus the positions returned from the
// wasm calls needs to be mapped between the two for editor integration.

/**
 * @param positions - An array of utf-8 code unit indexes to map to utf-16 code unit indexes
 * @param source - The source code to do the mapping on
 * @returns An object where the keys are the utf-8 index and the values are the utf-16 index
 */
export function mapUtf16UnitsToUtf8Units(
  positions: Array<number>,
  source: string,
): { [index: number]: number } {
  return mapStringIndexes(source, positions, "utf16");
}

/**
 * @param positions - An array of utf-8 code unit indexes to map to utf-16 code unit indexes
 * @param source - The source code to do the mapping on
 * @returns An object where the keys are the utf-8 index and the values are the utf-16 index
 */
export function mapUtf8UnitsToUtf16Units(
  positions: Array<number>,
  source: string,
): { [index: number]: number } {
  return mapStringIndexes(source, positions, "utf8");
}

function mapStringIndexes(
  buffer: string,
  indexes: Array<number>,
  sourceIndexType: "utf8" | "utf16",
): { [index: number]: number } {
  const result: { [index: number]: number } = {};
  if (indexes.length === 0) return result;

  // Remove any duplicates by converting to a set and back to an array
  const dedupedIndexes = [...new Set(indexes)];

  // Do one pass through the string, so ensure the indexes are in ascending order
  const sortedIndexes = dedupedIndexes.sort((a, b) => (a < b ? -1 : 1));

  // Assume that Rust handles utf-8 correctly in strings, and that the UTF-8 code units
  // per Unicode code point are as per the ranges below:
  // - 0x000000 to 0x00007F = 1 utf-8 code unit
  // - 0x000080 to 0x0007FF = 2 utf-8 code units
  // - 0x000800 to 0x00FFFF = 3 utf-8 code units
  // - 0x010000 to 0x10FFFF = 4 utf-8 code units
  //
  // Also assume the string is valid UTF-16 and all characters
  // outside the BMP (i.e. > 0xFFFF) are encoded with valid 'surrogate pairs', and
  // no other UTF-16 code units in the D800 - DFFF range occur.

  // A valid pair must be "high" surrogate (D800–DBFF) then "low" surrogates (DC00–DFFF)
  function isValidSurrogatePair(first: number, second: number): boolean {
    if (
      first < 0xd800 ||
      first > 0xdbff ||
      second < 0xdc00 ||
      second > 0xdfff
    ) {
      return false;
    }
    return true;
  }

  let utf16Index = 0;
  let utf8Index = 0;
  let sourceIndex = 0; // depending on the conversion requested, this will be equal to utf8Index or utf16Index
  let targetIndex = 0; // depending on the conversion requested, this will be equal to utf8Index or utf16Index
  let posArrayIndex = 0;
  let nextIndex = sortedIndexes[posArrayIndex];
  for (;;) {
    // Walk though the string, maintaining a UTF-8 to UTF-16 code unit index mapping.
    // When the string index >= the next searched for index, save that result and increment.
    // If the end of string or end of searched for indexes is reached, then break.
    if (sourceIndex >= nextIndex) {
      result[sourceIndex] = targetIndex;
      if (++posArrayIndex >= sortedIndexes.length) break;
      nextIndex = sortedIndexes[posArrayIndex];
    }

    if (utf16Index >= buffer.length) break;

    // Get the code unit (not code point) at the string index.
    const utf16CodeUnit = buffer.charCodeAt(utf16Index);

    // Advance the utf-8 index by the correct amount for the utf-16 code unit value.
    if (utf16CodeUnit < 0x80) {
      utf8Index += 1;
    } else if (utf16CodeUnit < 0x800) {
      utf8Index += 2;
    } else if (utf16CodeUnit < 0xd800 || utf16CodeUnit > 0xdfff) {
      // Not a surrogate pair, so one utf-16 code unit over 0x7FF == three utf-8 code utits
      utf8Index += 3;
    } else {
      // Need to consume the extra utf16 code unit for the pair also.
      const nextCodeUnit = buffer.charCodeAt(++utf16Index) || 0;
      if (!isValidSurrogatePair(utf16CodeUnit, nextCodeUnit))
        throw "Invalid surrogate pair";
      // Valid utf-16 surrogate pair implies code point over 0xFFFF implies 4 utf-8 code units.
      utf8Index += 4;
    }
    ++utf16Index; // Don't break here if EOF. We need to handle EOF being the final position to resolve.
    sourceIndex = sourceIndexType === "utf8" ? utf8Index : utf16Index;
    targetIndex = sourceIndexType === "utf8" ? utf16Index : utf8Index;
  }

  // TODO: May want to have a more configurable error reporting at some point. Avoid throwing here,
  // and just report and continue.
  if (posArrayIndex < sortedIndexes.length) {
    log.error(
      `Failed to map all ${sourceIndexType} indexes. Remaining indexes are: ${sortedIndexes.slice(
        posArrayIndex,
      )}`,
    );
  }

  return result;
}

export function mapDiagnostics(
  diags: VSDiagnostic[],
  code: string,
): VSDiagnostic[] {
  // Get a map of the Rust source positions to the JavaScript source positions
  const positions: number[] = [];
  diags.forEach((diag) => {
    positions.push(diag.start_pos);
    positions.push(diag.end_pos);
    diag.related?.forEach((related) => {
      positions.push(related.start_pos);
      positions.push(related.end_pos);
    });
  });
  const positionMap = mapUtf8UnitsToUtf16Units(positions, code);

  // Return the diagnostics with the positions mapped (or EOF if couldn't resolve)
  const results = diags.map((diag) => ({
    ...diag,
    // The mapped position may well be 0, so need to use ?? rather than ||
    start_pos: positionMap[diag.start_pos] ?? code.length,
    end_pos: positionMap[diag.end_pos] ?? code.length,
    related: diag.related?.map((related) => ({
      ...related,
      start_pos: positionMap[related.start_pos] ?? code.length,
      end_pos: positionMap[related.end_pos] ?? code.length,
    })),
  }));

  return results;
}
