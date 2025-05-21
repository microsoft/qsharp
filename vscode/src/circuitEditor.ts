// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { updateQsharpProjectContext } from "./debugger/activate";

export class CircuitEditorProvider implements vscode.CustomTextEditorProvider {
  private static readonly viewType = "qsharp-webview.circuit";
  updatingDocument: boolean = false;

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    const provider = new CircuitEditorProvider(context);
    const providerRegistration = vscode.window.registerCustomEditorProvider(
      CircuitEditorProvider.viewType,
      provider,
      { webviewOptions: { retainContextWhenHidden: true } },
    );
    return providerRegistration;
  }

  constructor(private readonly context: vscode.ExtensionContext) {}

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
  ): Promise<void> {
    // Setup initial content for the webview
    webviewPanel.webview.options = {
      enableScripts: true,
    };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    // Explicitly update context when this editor is resolved
    await updateQsharpProjectContext(document);

    // Also update context when the webview panel becomes active
    webviewPanel.onDidChangeViewState(async () => {
      if (webviewPanel.active) {
        await updateQsharpProjectContext(document);
      }
    });

    webviewPanel.webview.onDidReceiveMessage((e) => {
      switch (e.command) {
        case "update":
          this.updateTextDocument(document, e.text);
          return;
        case "read":
          updateWebview();
          return;
      }
    });

    const updateWebview = () => {
      const result = this.getDocumentAsJson(document);
      const filename = document.fileName.split(/\\|\//).pop()!.split(".")[0];

      if (result.error) {
        const message = {
          command: "error",
          props: {
            title: `${filename}`,
            message: result.error,
          },
        };
        webviewPanel.webview.postMessage(message);
        return;
      }

      const circuit = result.data;

      const props = {
        title: `${filename}`,
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

    // Update the webview when the text document changes
    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
      (event) => {
        if (event.document.uri.toString() === document.uri.toString()) {
          if (!this.updatingDocument && event.contentChanges.length > 0) {
            // Update the webview with the new document content
            updateWebview();
          }
        }
      },
    );

    // Dispose of the event listener when the webview is closed
    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
    });
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

  private getDocumentAsJson(document: vscode.TextDocument): {
    error?: string;
    data?: any;
  } {
    const text = document.getText();
    if (text.trim().length === 0) {
      return { data: {} };
    }

    try {
      return { data: JSON.parse(text) };
    } catch {
      return { error: "Content is not valid JSON" };
    }
  }

  private async updateTextDocument(
    document: vscode.TextDocument,
    circuit: string,
  ) {
    // Short-circuit if there are no changes to be made.
    if (
      circuit.trim().replace(/\r\n/g, "\n") ===
      document.getText().trim().replace(/\r\n/g, "\n")
    ) {
      return;
    }

    const edit = new vscode.WorkspaceEdit();

    edit.replace(
      document.uri,
      new vscode.Range(0, 0, document.lineCount, 0),
      circuit.trim(),
    );
    this.updatingDocument = true;
    await vscode.workspace.applyEdit(edit);
    this.updatingDocument = false;
  }
}
