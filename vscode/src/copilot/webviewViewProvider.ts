// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import {
  commands,
  CancellationToken,
  ExtensionContext,
  Uri,
  WebviewView,
  WebviewViewProvider,
  WebviewViewResolveContext,
  window,
} from "vscode";
import { Copilot, CopilotUpdateHandler } from "./copilot";
import { CopilotCommand } from "./shared";
import { qsharpExtensionId } from "../common";

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
  context.subscriptions.push(
    commands.registerCommand(`${qsharpExtensionId}.copilotClear`, async () => {
      provider.clearChat();
    }),
  );
}

class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {}

  private copilot: Copilot | undefined;
  private _streamCallback: CopilotUpdateHandler | undefined;

  resolveWebviewView(
    webviewView: WebviewView,
    context: WebviewViewResolveContext,
    token: CancellationToken,
  ): Thenable<void> | void {
    if (token.isCancellationRequested) return;

    this._streamCallback = ({ payload, kind }) => {
      if (this.view) {
        this.view.webview.postMessage({
          kind,
          payload,
        });
      }
    };

    this.view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.extensionUri],
    };

    try {
      this.copilot = new Copilot(this._streamCallback);

      const getUri = (pathList: string[]) =>
        webviewView.webview.asWebviewUri(
          Uri.joinPath(this.extensionUri, ...pathList),
        );

      const copilotJs = getUri(["out", "copilot", "webview", "copilot.js"]);
      const copilotCss = getUri(["out", "copilot", "webview", "copilot.css"]);
      const katexCss = getUri(["out", "katex", "katex.min.css"]);
      const codiconCss = getUri(["out", "katex", "codicon.css"]);
      const hljsCss = getUri(["out", "katex", "hljs-light.css"]);

      webviewView.webview.html = `<!DOCTYPE html>
        <html lang="en">
        <head>
        <link rel="stylesheet" href="${katexCss}" />
        <link rel="stylesheet" href="${codiconCss}" />
        <link rel="stylesheet" href="${hljsCss}" />
        <link rel="stylesheet" href="${copilotCss}" />
        </head>
        <body>
        <script src="${copilotJs}"></script>
        </body>
        </html>`;

      webviewView.webview.onDidReceiveMessage(
        this.handleMessageFromWebview.bind(this),
      );
    } catch (e) {
      log.error("Error loading Copilot: ", e);
      webviewView.webview.html = `<!DOCTYPE html>
        <html lang="en">
        <body>Error loading Copilot: ${e}</body>
        </html>`;
    }
  }

  clearChat() {
    this.copilot?.restartChat([]);
  }

  handleMessageFromWebview(message: CopilotCommand) {
    if (!this.copilot) {
      return;
    }

    switch (message.command) {
      case "submitUserMessage": {
        this.copilot.postUserMessage(message.request);
        break;
      }
      case "restartChat": {
        this.copilot.restartChat(message.history, message.service);
        break;
      }
    }
  }
}
