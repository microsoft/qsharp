# Building from Source

To install locally:

- Build the extension by running `build.py` or `build.py --wasm --npm --vscode` from the repo root.
- Package the `VSIX` with `vsce package` while in the `vscode` directory. To get `vsce`, run `npm install -g @vscode/vsce`
- In VS Code, run command "Extensions: Install from VSIX..."
- Select the `VSIX` you just packaged (`qsharp.vscode-0.0.0.vsix` for example) in the directory.
- Reload your VS Code window.

This will enable the extension for all instances of VS Code.

To scope the extension to only a specific workspace (for example, the `qsharp` repo):

- In VS Code, find and open the "Q# (new)" extension in the Extensions view.
- Click the "Disable" button to disable the extension globally.
- Click the dropdown next to "Enable" button and select "Enable (Workspace)".

This will enable the extension for only the current workspace. The extension will remain
enabled for that workspace across restarts.
