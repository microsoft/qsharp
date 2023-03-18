// Only compile this library for wasm targets
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// TODO: Below is an example of how to return typed structures from Rust via Wasm
// to the consuming JavaScript/TypeScript code. To be replaced with the implementation.

// These definitions match the values expected by VS Code and Monaco.
enum CompletionKind {
    Method = 1,
    Keyword = 13,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

// There is no easy way to serialize the result with serde_wasm_bindgen and get
// good TypeScript typing. Here we manually specify the type that the follow
// method will return. At the call-site in the TypeScript, the response should be
// cast to this type. (e.g., var result = get_completions() as ICompletionList).
// It does mean this type decl must be kept up to date with any structural changes.
#[wasm_bindgen(typescript_custom_section)]
const ICompletionList: &'static str = r#"
interface ICompletionList {
    items: Array<{
        label: string;
        kind: number;
    }>
}
"#;

#[wasm_bindgen]
pub fn get_completions() -> Result<JsValue, JsValue> {
    let res = CompletionList {items: vec![
        CompletionItem {label: "CCNOT".to_string(),       kind: CompletionKind::Method as i32},
        CompletionItem {label: "CNOT".to_string(),        kind: CompletionKind::Method as i32},
        CompletionItem {label: "CZ".to_string(),          kind: CompletionKind::Method as i32},
        CompletionItem {label: "X".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "Y".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "Z".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "H".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "S".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "T".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "M".to_string(),           kind: CompletionKind::Method as i32},
        CompletionItem {label: "CheckZero".to_string(),   kind: CompletionKind::Method as i32},
        CompletionItem {label: "DumpMachine".to_string(), kind: CompletionKind::Method as i32},
        CompletionItem {label: "Equal".to_string(),       kind: CompletionKind::Method as i32},
        CompletionItem {label: "Qubit".to_string(),       kind: CompletionKind::Method as i32},
        CompletionItem {label: "Reset".to_string(),       kind: CompletionKind::Method as i32},
        CompletionItem {label: "@EntryPoint".to_string(), kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "Adjoint".to_string(),     kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "Controlled".to_string(),  kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "Int".to_string(),         kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "if".to_string(),          kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "else".to_string(),        kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "namespace".to_string(),   kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "open".to_string(),        kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "operation".to_string(),   kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "return".to_string(),      kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "use".to_string(),         kind: CompletionKind::Keyword as i32},
        CompletionItem {label: "Unit".to_string(),        kind: CompletionKind::Keyword as i32},
    ]};
    Ok(serde_wasm_bindgen::to_value(&res)?)
}
