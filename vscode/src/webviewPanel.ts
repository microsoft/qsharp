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
  ExtensionContext,
  Uri,
  ViewColumn,
  Webview,
  WebviewPanel,
  window,
} from "vscode";
import { isQsharpDocument } from "./common";
import { reSampleData } from "./reSampleData";

const histogramRunTimeoutMs = 1000 * 60 * 5; // 5 minutes

export function registerWebViewCommands(context: ExtensionContext) {
  QSharpWebViewPanel.extensionUri = context.extensionUri;

  const compilerWorkerScriptPath = Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  // Stub for now to prototype the UX
  // Add the following to settings.json to enable the command:
  //   "Q#.experimental-re": true
  context.subscriptions.push(
    commands.registerCommand("qsharp-vscode.showRe", async () => {
      const message = {
        command: "estimate",
        estimatesData: reSampleData,
      };
      sendMessageToPanel("estimate", true, message);
    }),
  );

  context.subscriptions.push(
    commands.registerCommand("qsharp-vscode.showHelp", async () => {
      const message = {
        command: "help",
      };
      sendMessageToPanel("help", true, message);
    }),
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

      // Start the worker, run the code, and send the results to the webview
      const worker = getCompilerWorker(compilerWorkerScriptPath);
      const compilerTimeout = setTimeout(() => {
        worker.terminate(); // Confirm: Does the 'terminate' in the finally below error if this happens?
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

        sendMessageToPanel("histogram", true, undefined);

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
          sendMessageToPanel("histogram", false, message);
        });

        await worker.run(code, "", parseInt(numberOfShots), evtTarget);
        clearTimeout(compilerTimeout);
      } catch (e: any) {
        log.error("Codegen error. ", e.toString());
        throw new Error("Run failed");
      } finally {
        worker.terminate();
      }
    }),
  );
}

type PanelType = "histogram" | "estimate" | "help";

const panelTypeToPanel: Record<
  PanelType,
  { title: string; panel: QSharpWebViewPanel | undefined }
> = {
  histogram: { title: "Q# Histogram", panel: undefined },
  estimate: { title: "Q# Estimates", panel: undefined },
  help: { title: "Q# Help", panel: undefined },
};

function sendMessageToPanel(
  panelType: PanelType,
  reveal: boolean,
  message: any,
) {
  const panelRecord = panelTypeToPanel[panelType];
  if (!panelRecord.panel) {
    const panel = window.createWebviewPanel(
      panelType,
      panelRecord.title,
      ViewColumn.Beside,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
      },
    );

    panelRecord.panel = new QSharpWebViewPanel(panelType, panel);
  }

  if (reveal) panelRecord.panel.reveal(ViewColumn.Beside);
  if (message) panelRecord.panel.sendMessage(message);
}

export class QSharpWebViewPanel {
  public static extensionUri: Uri;
  private _ready = false;
  private _queuedMessages: any[] = [];

  constructor(
    private type: PanelType,
    private panel: WebviewPanel,
  ) {
    this.panel.onDidDispose(() => this.dispose());

    this.panel.webview.html = this._getWebviewContent(this.panel.webview);
    this._setWebviewMessageListener(this.panel.webview);
  }

  reveal(column: ViewColumn) {
    this.panel.reveal(column);
  }

  private _getWebviewContent(webview: Webview) {
    const extensionUri = QSharpWebViewPanel.extensionUri;

    function getUri(pathList: string[]) {
      return webview.asWebviewUri(Uri.joinPath(extensionUri, ...pathList));
    }

    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const githubCss = getUri(["out", "katex", "github-markdown.css"]);
    const webviewCss = getUri(["out", "webview", "webview.css"]);
    const webviewJs = getUri(["out", "webview", "webview.js"]);

    return /*html*/ `
  <!DOCTYPE html>
  <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Q#</title>
      <link rel="stylesheet" href="${githubCss}" />
      <link rel="stylesheet" href="${katexCss}" />
      <link rel="stylesheet" href="${webviewCss}" />
      <script src="${webviewJs}"></script>
    </head>
    <body>
    </body>
  </html>
`;
  }

  sendMessage(message: any) {
    if (this._ready) {
      log.debug("Sending message to webview", message);
      this.panel.webview.postMessage(message);
    } else {
      log.debug("Queuing message to webview", message);
      this._queuedMessages.push(message);
    }
  }

  private _setWebviewMessageListener(webview: Webview) {
    console.log("Setting up webview message listener");
    webview.onDidReceiveMessage((message: any) => {
      if (message.command === "ready") {
        this._ready = true;
        this._queuedMessages.forEach((message) =>
          this.panel.webview.postMessage(message),
        );
        this._queuedMessages = [];
      }

      // No messages are currently sent from the webview
      console.log("Message for webview received", message);
    });
  }

  public dispose() {
    panelTypeToPanel[this.type].panel = undefined;
    this.panel.dispose();
  }
}
