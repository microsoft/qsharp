# README

Note that as the WASM code is mostly just wrapping functionality from other crates
and returning a JsObject, there is little value in testing via `wasm-pack test`,
which has unit tests written in Rust. Wasm specific tests should be written in JavaScript
to run in the browser (or Node).
