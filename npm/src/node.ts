import {type ICompletionList, get_completions, check_code} from "../lib/node/qsc_wasm.cjs";

export function getCompletions() {
    let completions = get_completions() as ICompletionList;
    console.log(JSON.stringify(completions, null, 2));
}

export function checkCode(code: string) {
    let result = check_code(code);
    console.log(JSON.stringify(result, null, 2))
}
