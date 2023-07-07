# Testing

## VS Code for Web

To run the extension in a local browser environment, see
<https://code.visualstudio.com/api/extension-guides/web-extensions#test-your-web-extension-in-a-browser-using-vscodetestweb>

Basically, from the command-line in the repo root run:

```bash
npx @vscode/test-web --extensionDevelopmentPath ./vscode ./samples
```

This will open a Chromium browser hosting the VS Code for Web, with the extension
loaded from disk, and serving the samples directory as an open workspace.
