# Testing

## VS Code for Web

To run the extension in a local browser environment, see
<https://code.visualstudio.com/api/extension-guides/web-extensions#test-your-web-extension-in-a-browser-using-vscodetestweb>

From the `vscode` directory, run:

```bash
npm run run:web
```

To run from the repo root, you can pass the `-w` switch to `npm`:

```bash
 npm -w vscode run run:web
```

This will open a Chromium browser hosting the VS Code for Web, with the extension
loaded from disk, and serving the samples directory as an open workspace.

## Integration tests

To run the integration tests, from the `vscode` folder, run:

```bash
npm test
```

This will run the integration tests in an instance of VS Code for the Web.
See `test/runTests.mjs` and `test/suite/index.ts` for more details on how tests are
discovered and run.
