// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
    default as async_init, run, get_completions, check_code,
    type ICompletionList, type IDiagnostic
} from "../lib/web/qsc_wasm.js";

import { eventStringToMsg, run_shot_internal, type ShotResult } from "./common.js";

export async function init(wasm_uri: string) {
    let wasmBytes = await fetch(wasm_uri);
    await async_init(wasmBytes).then(wasm => {
        // TODO set_panic_hook
        console.log(`qsharp wasm module loaded from ${wasm_uri}`);
    });
}

export function getCompletions(): ICompletionList {
    let results = get_completions() as ICompletionList;
    return results;
}

export function checkCode(code: string): IDiagnostic[] {
    let result = check_code(code) as IDiagnostic[];
    return result;
}

export function evaluate(code: string, expr: string,
    eventCb: (msg: string) => void, shots: number): string {

    let result = run(code, expr, eventCb, shots) as string;
    return result;
}

export function run_shot(code: string, expr: string): ShotResult {
    return run_shot_internal(code, expr, run);
}

export { type IDiagnostic }
export { renderDump, exampleDump } from "./state-table.js"
export { outputAsDump, outputAsMessage, outputAsResult, eventStringToMsg, type Dump, type ShotResult } from "./common.js";
export { getAllKatas, getKata, getExercise, runExercise, type Kata, type KataModule, type Exercise } from "./katas.js";
