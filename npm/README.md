# qsharp npm module

This package contains the qsharp compiler functionality shipped for consumption via npm.

The source is written in TypeScript, which is compiled to ECMAScript modules in the ./dist directory.
The wasm binaries from the Rust builds are copied to the ./lib directory.

Consuming browser projects should import from this module and use a bundler to create their
own JavaScript bundle, and also copy the wasm file to their project and provide the URL
to it when calling the `init` method so it may be located and loaded.

Node.js tests can be run via `node --test` (see
<https://nodejs.org/dist/latest-v18.x/docs/api/test.html#test-runner-execution-model>).

The test module was also added to Node.js v16.17.0, and Electron 22 (which VS Code plans to move to
in first half of 2023) includes v16.17.1, so v16.17 should be our minimum Node.js
version supported (it shipped in Aug 2022).
