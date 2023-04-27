# Q# language VS Code Extension

Currently, this extension only contains the TextMate grammar and language configuration for Q#. 
It enables syntax highlighting and basic syntactic features (brace matching, etc.).

To install locally:
- In VS Code, run command "Developer: Install Extension from Location..."
- Select the `vscode` directory.

This will enable the extension for all instances of VS Code, applying syntax highlighting to
any .qs files that are opened.

To scope the extension to only a specific workspace (for example, the `qsharp` repo):
- In VS Code, find and open the "Q# (new)" extension in the Extensions view.
- Click the "Disable" button to disable the extension globally.
- Click the dropdown next to "Enable" button and select "Enable (Workspace)".

This will enable the extension for only the current workspace. The extension will remain
enabled for that workspace across restarts.