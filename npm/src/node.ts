// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {type ICompletionList, get_completions, check_code, run, IDiagnostic} from "../lib/node/qsc_wasm.cjs";

export function getCompletions() : ICompletionList {
    let completions = get_completions() as ICompletionList;
    return completions;
}

export function checkCode(code: string) : IDiagnostic[] {
    let result = check_code(code) as IDiagnostic[];
    return result;
}

export function evaluate(code: string, expr: string, cb?: Function) : string{
    // Last param is optional. Cast to any to supress error.
    let result = run(code, expr, cb as any) as string;
    return result;
}
