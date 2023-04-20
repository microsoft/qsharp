// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import async_init, * as wasm from "../lib/web/qsc_wasm.js";
import {Compiler} from "./compiler.js";
import { eventStringToMsg, mapDiagnostics, run_shot_internal, type ShotResult } from "./common.js";

export async function init(wasm_uri: string) {
    let wasmBytes = await fetch(wasm_uri);
    await async_init(wasmBytes).then(wasm => {
        // TODO set_panic_hook
        console.log(`qsharp wasm module loaded from ${wasm_uri}`);
    });
}

export function getCompiler(): Compiler {
    return new Compiler(wasm);
}

export function getCompletions(): wasm.ICompletionList {
    let results = wasm.get_completions() as wasm.ICompletionList;
    return results;
}

export function checkCode(code: string): wasm.IDiagnostic[] {
    let result = wasm.check_code(code) as wasm.IDiagnostic[];

    return mapDiagnostics(result, code);
}

export function evaluate(code: string, expr: string,
    eventCb: (msg: string) => void, shots: number): string {

    let result = wasm.run(code, expr, eventCb, shots) as string;
    return result;
}

export function run_shot(code: string, expr: string): ShotResult {
    return run_shot_internal(code, expr, wasm.run);
}

type IDiagnostic = wasm.IDiagnostic;

export type { IDiagnostic }
export { renderDump, exampleDump } from "./state-table.js"
export { outputAsDump, outputAsMessage, outputAsResult, eventStringToMsg, mapDiagnostics, type Dump, type ShotResult } from "./common.js";
export { getAllKatas, getKata, runExercise, type Kata, type KataItem, type Exercise } from "./katas.js";
