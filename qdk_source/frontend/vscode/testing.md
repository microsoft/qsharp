# Testing

## VS Code for Web

To run the extension in a local browser environment, see
<https://code.visualstudio.com/api/extension-guides/web-extensions#test-your-web-extension-in-a-browser-using-vscodetestweb>

From the `vscode` directory, run:

```bash
npm start
```

To run from the repo root, you can pass the `-w` switch to `npm`:

```bash
 npm -w vscode start
```

This will open a Chromium browser hosting the VS Code for Web, with the extension
loaded from disk, and serving the samples directory as an open workspace.

## Integration tests

To run the integration tests, from the `vscode` folder, run:

```bash
npm test
```

This will run the integration tests in an instance of VS Code for the Web.
See [test/runTests.mjs](test/runTests.mjs) and [test/suite/index.ts](test/runTests.mjs) for more details on how tests are
discovered and run.

## Debugging

To run a specific test suite under the debugger:

```bash
npm test -- --suite=language-service --waitForDebugger=1234
```

and attach a Chrome debugger (VS Code or Chrome/Edge Dev Tools) on port 1234.
