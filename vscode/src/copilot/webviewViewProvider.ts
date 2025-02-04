// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  CancellationToken,
  ExtensionContext,
  Uri,
  WebviewView,
  WebviewViewProvider,
  WebviewViewResolveContext,
  window,
} from "vscode";
import { CopilotCommand } from "./shared";
import { Copilot, CopilotUpdateHandler } from "./copilot";

export function registerCopilotPanel(context: ExtensionContext): void {
  const provider = new CopilotWebviewViewProvider(context.extensionUri);
  context.subscriptions.push(
    window.registerWebviewViewProvider(
      CopilotWebviewViewProvider.viewType,
      provider,
      {
        webviewOptions: { retainContextWhenHidden: true },
      },
    ),
  );
}

class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {
    this._streamCallback = ({ payload, kind }) => {
      if (this.view) {
        this.view.webview.postMessage({
          kind,
          payload,
        });
      }
    };

    this._copilot = new Copilot("AzureQuantumTest", this._streamCallback);
  }

  private _copilot: Copilot;
  private _streamCallback: CopilotUpdateHandler;

  resolveWebviewView(
    webviewView: WebviewView,
    context: WebviewViewResolveContext,
    token: CancellationToken,
  ): Thenable<void> | void {
    if (token.isCancellationRequested) return;

    this.view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.extensionUri],
    };

    const getUri = (pathList: string[]) =>
      webviewView.webview.asWebviewUri(
        Uri.joinPath(this.extensionUri, ...pathList),
      );

    const copilotJs = getUri(["out", "copilot", "webview", "copilot.js"]);
    const copilotCss = getUri(["out", "copilot", "webview", "copilot.css"]);
    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const codiconCss = getUri(["out", "katex", "codicon.css"]);

    webviewView.webview.html = `<!DOCTYPE html>
    <html lang="en">
    <head>
    <link rel="stylesheet" href="${katexCss}" />
    <link rel="stylesheet" href="${copilotCss}" />
    <link rel="stylesheet" href="${codiconCss}" />
    </head>
    <body>
    <script src="${copilotJs}"></script>
    </body>
    </html>`;

    webviewView.webview.onDidReceiveMessage(
      this.handleMessageFromWebview.bind(this),
    );
  }

  handleMessageFromWebview(message: CopilotCommand) {
    switch (message.command) {
      case "submitUserMessage": {
        this._copilot.postUserMessage(message.request);
        break;
      }
      case "restartChat": {
        this._copilot.restartChat(message.history, message.service);
        break;
      }
    }
  }
}
