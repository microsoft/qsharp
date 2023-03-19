import {default as async_init, 
    get_completions, type ICompletionList,
    check_code, type IDiagnostic
} from "../lib/web/qsc_wasm.js";

export async function init(wasm_uri: string) {
    let wasmBytes = await fetch(wasm_uri);
    await async_init(wasmBytes).then(wasm => {
        // TODO set_panic_hook
        console.log(`wasm module loaded from ${wasm_uri}`);
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
