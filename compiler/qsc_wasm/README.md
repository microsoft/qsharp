# README

Note that as the WASM code is mostly just wrapping functionality from other crates
and returning a JsObject, there is little value in testing via `wasm-pack test`,
which has unit tests written in Rust. Wasm specific tests should be written in JavaScript
to run in the browser (or Node).

The JavaScript wasm-pack spits out references `import.meta.url`.  esbuild may complain
about import.meta.url not being available. Should be fixable with:

--define:import.meta.url=document.URL
