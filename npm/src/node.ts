// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../lib/node/qsc_wasm.cjs";
import { Compiler } from "./compiler.js";
import { mapDiagnostics, run_shot_internal, ShotResult } from "./common.js";

export function getCompiler() : Compiler {
    return new Compiler(wasm);
}

export function getCompletions(): wasm.ICompletionList {
    let completions = wasm.get_completions() as wasm.ICompletionList;
    return completions;
}

export function checkCode(code: string): wasm.IDiagnostic[] {
    let result = wasm.check_code(code) as wasm.IDiagnostic[];
    return mapDiagnostics(result, code);
}

export function evaluate(code: string, expr: string, cb: Function, shots: number): string {
    let result = wasm.run(code, expr, cb, shots) as string;
    return result;
}

export function run_shot(code: string, expr: string): ShotResult {
    return run_shot_internal(code, expr, wasm.run);
}

export { mapDiagnostics } from "./common.js";
