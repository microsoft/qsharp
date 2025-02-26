// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { qsharpExtensionId } from "./common";
import { sampleCircuit } from "./sampleCircuit";

export class CircuitEditorProvider implements vscode.CustomTextEditorProvider {
  private static readonly viewType = "qsharp-webview.circuit";

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    log.info("Registering CircuitEditorProvider");

    context.subscriptions.push(
      vscode.commands.registerCommand(
        `${qsharpExtensionId}.createCircuitFile`,
        CircuitEditorProvider.createCircuitFile,
      ),
    );

    const provider = new CircuitEditorProvider(context);
    const providerRegistration = vscode.window.registerCustomEditorProvider(
      CircuitEditorProvider.viewType,
      provider,
    );
    return providerRegistration;
  }

  static async createCircuitFile(folderUri: vscode.Uri | undefined) {
    if (!folderUri) {
      // Try to detect the project folder of the currently active file
      // ToDo: This won't work for non-text files, such as the circuit files.
      const activeEditor = vscode.window.activeTextEditor;
      if (activeEditor) {
        console.log("Active editor found");
        let activeFileUri = activeEditor.document.uri;

        while (activeFileUri.path !== "/") {
          const parentUri = vscode.Uri.joinPath(activeFileUri, "..");
          const qsharpJsonPath = vscode.Uri.joinPath(parentUri, "qsharp.json");
          // Check if qsharp.json exists in the parent directory
          try {
            await vscode.workspace.fs.stat(qsharpJsonPath);
            folderUri = parentUri;
            break;
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
          } catch (_error) {
            activeFileUri = parentUri;
          }
        }
      }

      if (!folderUri) {
        console.log("No active editor found");
        // This was run from the command palette, not the context menu, so create the project
        // in the root of an open workspace folder.
        const workspaceCount = vscode.workspace.workspaceFolders?.length || 0;
        if (workspaceCount === 0) {
          // No workspaces open
          vscode.window.showErrorMessage(
            "You must have a folder open to create a Q# project in",
          );
          return;
        } else if (workspaceCount === 1) {
          folderUri = vscode.workspace.workspaceFolders![0].uri;
        } else {
          const workspaceChoice = await vscode.window.showWorkspaceFolderPick({
            placeHolder: "Pick the workspace folder for the project",
          });
          if (!workspaceChoice) return;
          folderUri = workspaceChoice.uri;
        }
      }
    }

    const defaultFileName = "NewCircuit";
    let fileName = await vscode.window.showInputBox({
      prompt: "Enter the name of the new circuit file",
      value: defaultFileName,
    });

    fileName = (fileName?.trim() || defaultFileName) + ".qviz";

    if (!fileName) {
      vscode.window.showErrorMessage("No file name provided.");
      return;
    }

    const fileUri = vscode.Uri.joinPath(folderUri, "src", fileName);
    const initialContent = JSON.stringify(sampleCircuit);
    const edit = new vscode.WorkspaceEdit();
    edit.createFile(fileUri);
    edit.set(fileUri, [
      new vscode.TextEdit(new vscode.Range(0, 0, 0, 0), initialContent),
    ]);

    // This doesn't throw on failure, it just returns false
    if (!(await vscode.workspace.applyEdit(edit))) {
      vscode.window.showErrorMessage(
        "Unable to create circuit file. Check the circuit file doesn't already exist and that the file system is writable",
      );
      return;
    }

    await vscode.commands.executeCommand(
      "vscode.openWith",
      fileUri,
      CircuitEditorProvider.viewType,
    );
  }

  constructor(private readonly context: vscode.ExtensionContext) {
    log.info("Constructing CircuitEditorProvider");
  }

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _token: vscode.CancellationToken,
  ): Promise<void> {
    log.info("Resolving CircuitEditorProvider");

    // Setup initial content for the webview
    webviewPanel.webview.options = {
      enableScripts: true,
    };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    webviewPanel.webview.onDidReceiveMessage((e) => {
      switch (e.command) {
        case "alert":
          vscode.window.showErrorMessage(e.text);
          return;
        case "update":
          this.updateTextDocument(document, e.text);
          return;
        case "read":
          updateWebview();
          return;
      }
    });

    const updateWebview = () => {
      const circuit = this.getDocumentAsJson(document);
      const filename = document.fileName.split(/\\|\//).pop()?.split(".")[0];

      const props = {
        title: `${filename} Circuit`,
        targetProfile: "",
        simulated: false,
        calculating: false,
        circuit,
      };

      const message = {
        command: "circuit",
        props,
      };
      webviewPanel.webview.postMessage(message);
    };

    updateWebview();
  }

  private getHtmlForWebview(webview: vscode.Webview): string {
    const extensionUri = this.context.extensionUri;

    function getUri(pathList: string[]) {
      return webview.asWebviewUri(
        vscode.Uri.joinPath(extensionUri, ...pathList),
      );
    }

    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const githubCss = getUri(["out", "katex", "github-markdown-dark.css"]);
    const webviewCss = getUri(["out", "webview", "webview.css"]);
    const scriptUri = getUri(["out", "webview", "editor.js"]);
    const resourcesUri = getUri(["resources"]);
    return `
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>Q#</title>
          <link rel="stylesheet" href="${githubCss}" />
          <link rel="stylesheet" href="${katexCss}" />
          <link rel="stylesheet" href="${webviewCss}" />
          <script src="${scriptUri}"></script>
          <script>
            window.resourcesUri = "${resourcesUri.toString()}";
          </script>
        </head>
        <body>
        </body>
      </html>`;
  }

  private getDocumentAsJson(document: vscode.TextDocument): any {
    const text = document.getText();
    if (text.trim().length === 0) {
      return {};
    }

    try {
      return JSON.parse(text);
    } catch {
      throw new Error(
        "Could not get document as json. Content is not valid json",
      );
    }
  }

  private updateTextDocument(document: vscode.TextDocument, circuit: string) {
    // Short-circuit if there are no changes to be made.
    if (circuit == document.getText()) {
      return;
    }

    const edit = new vscode.WorkspaceEdit();

    // Just replace the entire document every time for this example extension.
    // A more complete extension should compute minimal edits instead.
    edit.replace(
      document.uri,
      new vscode.Range(0, 0, document.lineCount, 0),
      circuit,
    );

    return vscode.workspace.applyEdit(edit);
  }
}
