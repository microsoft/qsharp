# qsharp npm module

This package contains the qsharp compiler functionality shipped for consumption via npm.

The source is written in TypeScript, which is compiled to ECMAScript modules in the ./dist directory.
The wasm binaries from the Rust builds are copied to the ./lib directory.

Consuming browser projects should import from this module and use a bundler to create their
own JavaScript bundle, and also copy the wasm file to their project and provide the URL
to it when calling the `loadWasmModule` method so it may be located and loaded.

## Node and browser support

wasm-pack generates different files for the browser and Node.js environments. The wasm is slightly
different, and the loader code is quite different. This can be seen in `./lib/web/qsc_wasm.cjs`
and `./lib/node/qsc_wasm.js` files respectively. Specifically, the web environment loads the wasm
file using async web APIs such as `fetch` with a URI, and Node.js uses `require` to load the `fs` module
and calls to `readFileSync`. Once the wasm module is loaded however, the exported APIs are used
in a similar manner.

To support using this npm package from both environments, the package uses "conditional exports"
<https://nodejs.org/dist/latest-v18.x/docs/api/packages.html#conditional-exports> to expose one
entry point for Node.js, and another for browsers. The distinct entry points uses their respective
loader to load the wasm module for the platform, and then expose functionality that uses the
loaded module via common code.

When bundling for the web, bundlers such as esbuild will automatically use the default entry point,
whereas when loaded as a Node.js module, it will use the "node" entry point.

Note that TypeScript seems to add the ['import', 'types', 'node'] conditions by default when
searching the Node.js `exports`, and so will always find the 'node' export before the 'default'
export. To resolve this, a 'browser' condition was added (which is same as 'default' but earlier
than 'node') and the tsconfig compiler option `"customConditions": ["browser"]` should be added
(requires TypeScript 5.0 or later). esbuild also adds the 'browser' condition when bundling for
the browser (see <https://esbuild.github.io/api/#how-conditions-work>).

## Design

The API for using this module is similar whether using a browser or Node.js, and whether running
in the main thread or a worker thread. You instantiate the compiler, and call operations on it
which complete in the order called.

All operations return a Promise which resolves then the operation is complete. Some operations
may also emit events, such as debug messages or state dumps as they are processed. A call may
also be passed a CancellationToken so that if the result is no longer needed then the operation
may be cancelled before starting.

If the caller is not interested in any interim events or being able to cancel the request,
these arguments are optional. For example:

```js
const codeSample = "namespace Test { operation Main() {....} }";
const entryPoint = "Test.Main()";

const compiler  = getCompilerWorker();
const cancelSrc = new CancellationTokenSource();
const runEvents = new QscEventTarget(false /* store record of events */);

// Log any DumpMachine calls
runEvents.addEventListener('DumpMachine', (evt) => console.log("DumpMachine: %o", evt.detail));

compiler.run(codeSample, entryPoint, 1 /* shots */, runEvents, cancelSrc.token)
        .then(result => console.log("Run result: %s", result));
        .catch(err => console.err("Run failed with: %o", err));

cancelButton.addEventListener('click', () => {
    // Try to cancel the operation if still pending
    tokenSource.cancel();
});

// Also run the below request, but don't care about events or cancellation, just the result
const checkResult = await compiler.check(code);
console.log('check result was: %o', checkResult);
```

Promises, Events, and Cancellation are based on JavaScript or Web standards, or the VS Code API:

- Promises <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises>
- EventTarget <https://developer.mozilla.org/en-US/docs/Web/API/EventTarget>
- Event <https://developer.mozilla.org/en-US/docs/Web/API/Event/Event>
- VS Code API for CancellationToken <https://code.visualstudio.com/api/references/vscode-api#CancellationToken>

The standard Web APIs for custom events were added to Node.js in v16.17. <https://nodejs.org/dist/v16.17.0/docs/api/events.html>, but behind an experimental flag. As CustomEvent is not on
the global by default until v19 or later, the code will use Event with a 'detail'
property manually set until v20 is in common use.

The VS Code implementation for cancellation tokens is viewable in their source code
at <src/vs/base/common/cancellation.ts>. This code uses a simplified version of that API.

## Testing

Node.js tests can be run via `node --test` (see
<https://nodejs.org/dist/latest-v18.x/docs/api/test.html#test-runner-execution-model>).

The test module was also added to Node.js v16.17.0, and Electron 22 (which VS Code plans to move to
in first half of 2023) includes v16.17.1, so v16.17 should be our minimum Node.js
version supported (it shipped in Aug 2022).
