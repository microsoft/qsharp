# qsharp npm module

This package contains the qsharp compiler functionality shipped for consumption via npm.

The source is written in TypeScript, which is compiled to ECMAScript modules in the ./dist directory.
The wasm binaries from the Rust build is copied to the ./lib directory.

Consuming projects should import from this module and use a bundler to create their own JavaScript bundle.

Consuming projects will also need to copy the wasm file to their project and provide the URL to it
when calling the `init` method so it may be located and loaded.

To run the Node.js tests, Node.js version 18.1 (released May 2022) or later is required
(see <https://nodejs.org/dist/latest-v18.x/docs/api/test.html>).

Tests can be run via `node --test` (see <https://nodejs.org/dist/latest-v18.x/docs/api/test.html#test-runner-execution-model>).
