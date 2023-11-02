// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  QscEventTarget,
  VSDiagnostic,
  getCompilerWorker,
  log,
} from "qsharp-lang";
import {
  commands,
  Disposable,
  ExtensionContext,
  Uri,
  ViewColumn,
  Webview,
  WebviewPanel,
  window,
} from "vscode";
import { isQsharpDocument } from "./common";

const histogramRunTimeoutMs = 1000 * 60 * 5; // 5 minutes

export function registerHistogramCommand(context: ExtensionContext) {
  const compilerWorkerScriptPath = Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js"
  ).toString();

  context.subscriptions.push(
    commands.registerCommand("qsharp-vscode.showEstimates", async () => {
      const message = {
        command: "estimate",
      };
      QSharpWebViewPanel.render(context.extensionUri);
      QSharpWebViewPanel.currentPanel?.sendMessage(message);
    })
  );

  context.subscriptions.push(
    commands.registerCommand("qsharp-vscode.showHistogram", async () => {
      function resultToLabel(result: string | VSDiagnostic): string {
        if (typeof result !== "string") return "ERROR";
        return result;
      }

      const editor = window.activeTextEditor;
      if (!editor || !isQsharpDocument(editor.document)) {
        throw new Error("The currently active window is not a Q# file");
      }

      QSharpWebViewPanel.render(context.extensionUri);

      // Start the worker, run the code, and send the results to the webview
      const worker = getCompilerWorker(compilerWorkerScriptPath);
      const compilerTimeout = setTimeout(() => {
        worker.terminate(); // TODO: Does the 'terminate' in the finally below error if this happens?
      }, histogramRunTimeoutMs);
      try {
        const code = editor.document.getText();

        const validateShotsInput = (input: string) => {
          const result = parseFloat(input);
          if (isNaN(result) || Math.floor(result) !== result || result <= 0) {
            return "Number of shots must be a positive integer";
          }
        };

        const numberOfShots =
          (await window.showInputBox({
            value: "100",
            prompt: "Number of shots",
            validateInput: validateShotsInput,
          })) || "100";

        // abort if the user hits <Esc> during shots entry
        if (numberOfShots === undefined) {
          return;
        }

        const evtTarget = new QscEventTarget(true);
        evtTarget.addEventListener("uiResultsRefresh", () => {
          const results = evtTarget.getResults();
          const resultCount = evtTarget.resultCount();
          const buckets = new Map();
          for (let i = 0; i < resultCount; ++i) {
            const key = results[i].result;
            const strKey = resultToLabel(key);
            const newValue = (buckets.get(strKey) || 0) + 1;
            buckets.set(strKey, newValue);
          }
          const message = {
            command: "update",
            buckets: Array.from(buckets.entries()),
            shotCount: resultCount,
          };
          QSharpWebViewPanel.currentPanel?.sendMessage(message);
        });

        await worker.run(code, "", parseInt(numberOfShots), evtTarget);
        clearTimeout(compilerTimeout);
      } catch (e: any) {
        log.error("Codegen error. ", e.toString());
        throw new Error("Run failed");
      } finally {
        worker.terminate();
      }
    })
  );
}

function getUri(webview: Webview, extensionUri: Uri, pathList: string[]) {
  return webview.asWebviewUri(Uri.joinPath(extensionUri, ...pathList));
}

export class QSharpWebViewPanel {
  public static currentPanel: QSharpWebViewPanel | undefined;
  private readonly _panel: WebviewPanel;
  private _ready = false;
  private _queuedMessages: any[] = [];
  private _disposables: Disposable[] = [];

  private constructor(panel: WebviewPanel, extensionUri: Uri) {
    this._panel = panel;
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    this._panel.webview.html = this._getWebviewContent(
      this._panel.webview,
      extensionUri
    );
    this._setWebviewMessageListener(this._panel.webview);
  }

  private _getWebviewContent(webview: Webview, extensionUri: Uri) {
    const webviewCss = getUri(webview, extensionUri, [
      "resources",
      "webview.css",
    ]);
    const mathjaxJs = getUri(webview, extensionUri, [
      "out",
      "mathjax",
      "tex-chtml.js",
    ]);
    const webviewJs = getUri(webview, extensionUri, [
      "out",
      "webview",
      "webview.js",
    ]);

    return /*html*/ `
  <!DOCTYPE html>
  <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Q#</title>
      <link rel="stylesheet" href="${webviewCss}" />
      <script>
window.MathJax = {
  loader: {load: ['[tex]/physics']},
  tex: {
    packages: { "[+]": ["physics"] },
    inlineMath: [['$', '$']]
  }
};
      </script>
      <script src="${webviewJs}"></script>
      <script type="text/javascript" id="MathJax-script" async src="${mathjaxJs}"></script>
    </head>
    <body>
    </body>
  </html>
`;
  }

  sendMessage(message: any) {
    if (this._ready) {
      console.log("Sending message to webview", message);
      this._panel.webview.postMessage(message);
    } else {
      console.log("Queuing message to webview", message);
      this._queuedMessages.push(message);
    }
  }

  private _setWebviewMessageListener(webview: Webview) {
    console.log("Setting up webview message listener");
    webview.onDidReceiveMessage(
      (message: any) => {
        if (message.command === "ready") {
          this._ready = true;
          this._queuedMessages.forEach((message) =>
            this._panel.webview.postMessage(message)
          );
          this._queuedMessages = [];
        }

        // No messages are currently sent from the webview
        console.log("Message for webview received", message);
      },
      undefined,
      this._disposables
    );
  }

  public static render(extensionUri: Uri) {
    if (QSharpWebViewPanel.currentPanel) {
      // If the webview panel already exists reveal it
      QSharpWebViewPanel.currentPanel._panel.reveal(ViewColumn.Beside);
    } else {
      // If a webview panel does not already exist create and show a new one
      console.log("Creating new webview panel");
      const panel = window.createWebviewPanel(
        "qsharpWebView",
        "Q#",
        ViewColumn.Beside,
        {
          enableScripts: true,
          retainContextWhenHidden: true,
        }
      );

      QSharpWebViewPanel.currentPanel = new QSharpWebViewPanel(
        panel,
        extensionUri
      );
    }
  }

  public dispose() {
    QSharpWebViewPanel.currentPanel = undefined;

    // Dispose of the current webview panel
    this._panel.dispose();

    // Dispose of all disposables (i.e. commands) for the current webview panel
    while (this._disposables.length) {
      const disposable = this._disposables.pop();
      if (disposable) {
        disposable.dispose();
      }
    }
  }
}
