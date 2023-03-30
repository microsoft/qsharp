// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {default as async_init, run,
    get_completions, type ICompletionList,
    check_code, type IDiagnostic
} from "../lib/web/qsc_wasm.js";

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

export function evaluate(code: string, expr: string, eventCb?: Function) : string{
    // The last param is optional. Cast to any to supress error.
    let result = run(code, expr, eventCb as any) as string;
    return result;
}

export {type IDiagnostic}
export {renderDump, exampleDump} from "./state-table.js"
export {outputAsDump, outputAsMessage, type Dump} from "./common.js";
