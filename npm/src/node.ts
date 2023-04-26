// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type ICompletionList, get_completions, check_code, run, IDiagnostic } from "../lib/node/qsc_wasm.cjs";
import { mapDiagnostics, run_shot_internal, ShotResult } from "./common.js";

export function getCompletions(): ICompletionList {
    let completions = get_completions() as ICompletionList;
    return completions;
}

export function checkCode(code: string): IDiagnostic[] {
    let result = check_code(code) as IDiagnostic[];
    return mapDiagnostics(result, code);
}

export function evaluate(code: string, expr: string, cb: Function, shots: number): string {
    let result = run(code, expr, cb, shots) as string;
    return result;
}

export function run_shot(code: string, expr: string): ShotResult {
    return run_shot_internal(code, expr, run);
}

export { mapDiagnostics } from "./common.js";
