# README

This crate is only built if the 'wasm' feature is enabled. You may want to edit your 
VS Code settings to enable this feature by default to get good editor support while 
working on it, e.g. in settings.json:

```text
    "rust-analyzer.cargo.features": "all"
```

Note: If you explicitly enable the 'wasm' feature in the setting, then rust-analyzer fails
when trying to debug unit tests directly in the editor for projects that don't have a
'wasm' feature (which is most of them). Enabling 'all' features resolves this issue.

## TODOs

Note that as the WASM code is mostly just wrapping functionality from other crates
and returning a JsObject, there is little value in testing via `wasm-pack test`,
which has unit tests written in Rust. Wasm specific tests should be written in JavaScript
to run in the browser (or Node).
