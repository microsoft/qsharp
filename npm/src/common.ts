// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Each DumpMachine output is represented as an object where each key is a basis
// state, e.g., "|3>" and the value is the [real, imag] parts of the complex amplitude.
export type Dump = {
    [index: string]: [number, number];
};

export interface VSDiagnostic {
    start_pos: number;
    end_pos: number;
    message: string;
    severity: number;
}

export type Result = 
    { success: true, value: string } |
    { success: false, value: VSDiagnostic };

export interface DumpMsg {
    type: "DumpMachine";
    state: Dump;
}

export interface MessageMsg {
    type: "Message";
    message: string;
}

export interface ResultMsg {
    type: "Result";
    result: Result;
}

export type EventMsg = ResultMsg | DumpMsg | MessageMsg;

export function outputAsResult(msg: string) : ResultMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "Result" && typeof obj.success == "boolean") {
            return {
                type: "Result",
                result: {
                    success: obj.success,
                    value: obj.result
                }
            };
        }
    } catch {
        return null;
    }
    return null;
}

export function outputAsMessage(msg: string) : MessageMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "Message" && typeof obj.message == "string") {
            return obj as MessageMsg;
        }
    } catch {
        return null;
    }
    return null;
}

export function outputAsDump(msg: string) : DumpMsg | null {
    try {
        let obj = JSON.parse(msg);
        if (obj?.type == "DumpMachine" && typeof obj.state == "object") {
            return obj as DumpMsg;
        }
    } catch {
        return null;
    }
    return null;
}

export function eventStringToMsg(msg: string) : EventMsg | null {
    return outputAsResult(msg) || outputAsMessage(msg) || outputAsDump(msg);
}

export type ShotResult = {
    success: boolean;
    result: string | VSDiagnostic;
    events: Array<MessageMsg | DumpMsg>;
}

// The QSharp compiler returns positions in utf-8 code unit positions (basically a byte[]
// index), however VS Code and Monaco handle positions as utf-16 code unit positions
// (basically JavaScript string index positions). Thus the positions returned from the 
// wasm calls needs to be mapped between the two for editor integration.

/**
 * @param positions - An array of utf-8 code unit indexes to map to utf-16 code unit indexes
 * @param source - The source code to do the mapping on
 * @returns An object where the keys are the utf-8 index and the values are the utf-16 index
 */
export function mapUtf8UnitsToUtf16Units(positions: Array<number>, source: string) : {[index: number]: number} {
    let result : {[index: number]: number} = {};
    if (positions.length === 0) return result;

    // Remove any duplicates by converting to a set and back to an array
    const deduped_pos = [...new Set(positions)];

    // Do one pass through the source, so ensure the indexs are in ascending order
    const sorted_pos = deduped_pos.sort( (a, b) => (a < b ? -1 : 1));

    // Assume that Rust handles utf-8 correctly in strings, and that the UTF-8 code units 
    // per Unicode code point are as per the ranges below:
    // - 0x000000 to 0x00007F = 1 utf-8 code unit
    // - 0x000080 to 0x0007FF = 2 utf-8 code units
    // - 0x000800 to 0x00FFFF = 3 utf-8 code units
    // - 0x010000 to 0x10FFFF = 4 utf-8 code units
    //
    // Also assume the source JavaScript string is valid UTF-16 and all characters
    // outside the BMP (i.e. > 0xFFFF) are encoded with valid 'surrogate pairs', and
    // no other UTF-16 code units in the D800 - DFFF range occur.

    // A valid pair must be "high" surrogate (D800–DBFF) then "low" surrogates (DC00–DFFF)
    function isValidSurrogatePair(first: number, second: number) : boolean {
        if (first < 0xD800 || first > 0xDBFF || second < 0xDC00 || second > 0xDFFF) {
            return false;
        }
        return true;
    }

    let utf16Index = 0;
    let utf8Index = 0;
    let posArrayIndex = 0;
    let nextUtf8Target = sorted_pos[posArrayIndex];
    while (true) {
        // Walk though the source code maintaining a UTF-8 to UTF-16 code unit index mapping.
        // When the UTF-8 index >= the next searched for index, save that result and increment.
        // If the end of source or end of searched for positions is reached, then break
        if (utf8Index >= nextUtf8Target) {
            result[utf8Index] = utf16Index;
            if (++posArrayIndex >= sorted_pos.length) break;
            nextUtf8Target = sorted_pos[posArrayIndex];
        }

        if (utf16Index >= source.length) break;

        // Get the code unit (not code point) at the source index.
        const utf16CodeUnit = source.charCodeAt(utf16Index);

        // Advance the utf-8 offset by the correct amount for the utf-16 code unit value.
        if (utf16CodeUnit < 0x80) {
            utf8Index += 1;
        } else if (utf16CodeUnit < 0x800) {
            utf8Index += 2;
        } else if (utf16CodeUnit < 0xD800 || utf16CodeUnit > 0xDFFF) {
            // Not a surrogate pair, so one utf-16 code unit over 0x7FF == three utf-8 code utits
            utf8Index += 3;
        } else {
            // Need to consume the extra utf16 code unit for the pair also.
            const nextCodeUnit = source.charCodeAt(++utf16Index) || 0;
            if (!isValidSurrogatePair(utf16CodeUnit, nextCodeUnit)) throw "Invalid surrogate pair";
            // Valid utf-16 surrogate pair implies code point over 0xFFFF implies 4 utf-8 code units.
            utf8Index += 4;
        }
        ++utf16Index; // Don't break here if EOF. We need to handle EOF being the final position to resolve.
    }

    // TODO: May want to have a more configurable error reporting at some point. Avoid throwing here,
    // and just report and continue.
    if (posArrayIndex < sorted_pos.length) {
        console.error(`Failed to map all utf-8 positions to source locations. Remaining positions are: ${sorted_pos.slice(posArrayIndex)}`);
    }

    return result;
}

export function mapDiagnostics(diags: VSDiagnostic[], code: string) : VSDiagnostic[] {
    // Get a map of the Rust source positions to the JavaScript source positions
    const positions: number[] = [];
    diags.forEach(diag => {
        positions.push(diag.start_pos);
        positions.push(diag.end_pos);
    });
    const positionMap = mapUtf8UnitsToUtf16Units(positions, code);

    // Return the diagnostics with the positions mapped (or EOF if couldn't resolve)
    const results = diags.map(diag => ({
        ...diag,
        // The mapped position may well be 0, so need to use ?? rather than ||
        start_pos: positionMap[diag.start_pos] ?? code.length,
        end_pos: positionMap[diag.end_pos] ?? code.length,
    }));

    return results;
}
