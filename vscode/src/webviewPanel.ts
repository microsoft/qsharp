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
  WebviewPanelSerializer,
  window,
} from "vscode";
import { isQsharpDocument } from "./common";
import { loadProject } from "./projectSystem";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";

const QSharpWebViewType = "qsharp-webview";
const compilerRunTimeoutMs = 1000 * 60 * 5; // 5 minutes

export function registerWebViewCommands(context: ExtensionContext) {
  QSharpWebViewPanel.extensionUri = context.extensionUri;

  window.registerWebviewPanelSerializer(
    QSharpWebViewType,
    new QSharpViewViewPanelSerializer(),
  );

  const compilerWorkerScriptPath = Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  context.subscriptions.push(
    commands.registerCommand("qsharp-vscode.showRe", async () => {
      const associationId = getRandomGuid();
      sendTelemetryEvent(
        EventType.TriggerResourceEstimation,
        { associationId },
        {},
      );
      const editor = window.activeTextEditor;
      if (!editor || !isQsharpDocument(editor.document)) {
        throw new Error("The currently active window is not a Q# file");
      }

      const qubitType = await window.showQuickPick(
        [
          {
            label: "qubit_gate_ns_e3",
            detail: "Superconducting/spin qubit with 1e-3 error rate",
            picked: true,
            params: {
              qubitParams: { name: "qubit_gate_ns_e3" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_gate_ns_e4",
            detail: "Superconducting/spin qubit with 1e-4 error rate",
            params: {
              qubitParams: { name: "qubit_gate_ns_e4" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_gate_us_e3",
            detail: "Trapped ion qubit with 1e-3 error rate",
            params: {
              qubitParams: { name: "qubit_gate_us_e3" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_gate_us_e4",
            detail: "Trapped ion qubit with 1e-4 error rate",
            params: {
              qubitParams: { name: "qubit_gate_us_e4" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_maj_ns_e4 + surface_code",
            detail: "Majorana qubit with 1e-4 error rate (surface code QEC)",
            params: {
              qubitParams: { name: "qubit_maj_ns_e4" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_maj_ns_e6 + surface_code",
            detail: "Majorana qubit with 1e-6 error rate (surface code QEC)",
            params: {
              qubitParams: { name: "qubit_maj_ns_e6" },
              qecScheme: { name: "surface_code" },
            },
          },
          {
            label: "qubit_maj_ns_e4 + floquet_code",
            detail: "Majorana qubit with 1e-4 error rate (floquet code QEC)",
            params: {
              qubitParams: { name: "qubit_maj_ns_e4" },
              qecScheme: { name: "floquet_code" },
            },
          },
          {
            label: "qubit_maj_ns_e6 + floquet_code",
            detail: "Majorana qubit with 1e-6 error rate (floquet code QEC)",
            params: {
              qubitParams: { name: "qubit_maj_ns_e6" },
              qecScheme: { name: "floquet_code" },
            },
          },
        ],
        {
          canPickMany: true,
          title: "Qubit types",
          placeHolder: "Superconducting/spin qubit with 1e-3 error rate",
          matchOnDetail: true,
        },
      );

      if (!qubitType) {
        return;
      }

      // Prompt for error budget (default to 0.001)
      const validateErrorBudget = (input: string) => {
        const result = parseFloat(input);
        if (isNaN(result) || result <= 0.0 || result >= 1.0) {
          return "Error budgets must be between 0 and 1";
        }
      };

      const errorBudget = await window.showInputBox({
        value: "0.001",
        prompt: "Error budget",
        validateInput: validateErrorBudget,
      });

      // abort if the user hits <Esc> during shots entry
      if (errorBudget === undefined) {
        return;
      }

      // use document uri path to get the project name, since it is normalized to `/` separators
      // see https://code.visualstudio.com/api/references/vscode-api#Uri for difference between
      // path and fsPath
      const projectName =
        editor.document.uri.path.split("/").pop()?.split(".")[0] || "program";

      let runName = await window.showInputBox({
        title: "Friendly name for run",
        value: `${projectName}`,
      });
      if (!runName) {
        return;
      }

      const params = qubitType.map((item) => ({
        ...item.params,
        errorBudget: parseFloat(errorBudget),
        estimateType: "frontier",
      }));

      log.info("RE params", params);

      sendMessageToPanel("estimates", true, {
        command: "estimates",
        calculating: true,
      });

      // Ensure the name is unique
      if (panelTypeToPanel["estimates"].state[runName] !== undefined) {
        let idx = 2;
        for (;;) {
          const newName = `${runName}-${idx}`;
          if (panelTypeToPanel["estimates"].state[newName] === undefined) {
            runName = newName;
            break;
          }
          idx++;
        }
      }
      panelTypeToPanel["estimates"].state[runName] = true;

      // Start the worker, run the code, and send the results to the webview
      log.debug("Starting resource estimates worker.");
      let timedOut = false;

      const worker = getCompilerWorker(compilerWorkerScriptPath);
      const compilerTimeout = setTimeout(() => {
        log.info("Compiler timeout. Terminating worker.");
        timedOut = true;
        worker.terminate();
      }, compilerRunTimeoutMs);

      try {
        const sources = await loadProject(editor.document.uri);
        const start = performance.now();
        sendTelemetryEvent(
          EventType.ResourceEstimationStart,
          { associationId },
          {},
        );
        const estimatesStr = await worker.getEstimates(
          sources,
          JSON.stringify(params),
        );
        sendTelemetryEvent(
          EventType.ResourceEstimationEnd,
          { associationId },
          { timeToCompleteMs: performance.now() - start },
        );
        log.debug("Estimates result", estimatesStr);

        // Should be an array of one ReData object returned
        const estimates = JSON.parse(estimatesStr);

        for (const item of estimates) {
          // if item doesn't have a status property, it's an error
          if (!("status" in item) || item.status !== "success") {
            log.error("Estimates error code: ", item.code);
            log.error("Estimates error message: ", item.message);
            throw item.message;
          }
        }

        (estimates as Array<any>).forEach(
          (item, idx) =>
            (item.jobParams.runName = `${runName} (${String.fromCharCode(
              0x61 + idx,
            )})`),
        );

        clearTimeout(compilerTimeout);

        const message = {
          command: "estimates",
          calculating: false,
          estimates,
        };
        sendMessageToPanel("estimates", true, message);
      } catch (e: any) {
        // Stop the 'calculating' animation
        const message = {
          command: "estimates",
          calculating: false,
          estimates: [],
        };
        sendMessageToPanel("estimates", false, message);

        if (timedOut) {
          // Show a VS Code popup that a timeout occurred
          window.showErrorMessage(
            "The resource estimation timed out. Please try again.",
          );
        } else {
          log.error("getEstimates error: ", e.toString());
          throw new Error("Estimating failed with error: " + e.toString());
        }
      } finally {
        if (!timedOut) {
          log.debug("Terminating resource estimates worker.");
          worker.terminate();
        }
      }
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
      const associationId = getRandomGuid();
      sendTelemetryEvent(EventType.TriggerHistogram, { associationId }, {});
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
      }, compilerRunTimeoutMs);
      try {
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
            command: "histogram",
            buckets: Array.from(buckets.entries()),
            shotCount: resultCount,
          };
          sendMessageToPanel("histogram", false, message);
        });
        const sources = await loadProject(editor.document.uri);
        const start = performance.now();
        sendTelemetryEvent(EventType.HistogramStart, { associationId }, {});
        await worker.run(sources, "", parseInt(numberOfShots), evtTarget);
        sendTelemetryEvent(
          EventType.HistogramEnd,
          { associationId },
          { timeToCompleteMs: performance.now() - start },
        );
        clearTimeout(compilerTimeout);
      } catch (e: any) {
        log.error("Histogram error. ", e.toString());
        throw new Error("Run failed");
      } finally {
        worker.terminate();
      }
    }),
  );
}

type PanelType = "histogram" | "estimates" | "help";

const panelTypeToPanel: Record<
  PanelType,
  { title: string; panel: QSharpWebViewPanel | undefined; state: any }
> = {
  histogram: { title: "Q# Histogram", panel: undefined, state: {} },
  estimates: { title: "Q# Estimates", panel: undefined, state: {} },
  help: { title: "Q# Help", panel: undefined, state: {} },
};

function sendMessageToPanel(
  panelType: PanelType,
  reveal: boolean,
  message: any,
) {
  const panelRecord = panelTypeToPanel[panelType];
  if (!panelRecord.panel) {
    const panel = window.createWebviewPanel(
      QSharpWebViewType,
      panelRecord.title,
      {
        viewColumn: ViewColumn.Three,
        preserveFocus: true,
      },
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        // Note: If retainContextWhenHidden is false, the webview gets reloaded
        // every time you hide it by switching to another tab and then switch
        // back. While we've done the work to persist the underlying state, the
        // dynamic state of the DOM on the page - such as which detail sections are
        // expanded, whether in summary or detail view, etc. - is lost if this occurs.
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
    log.info("Creating webview panel of type", type);
    this.panel.onDidDispose(() => this.dispose());

    this.panel.webview.html = this._getWebviewContent(this.panel.webview);
    this._setWebviewMessageListener(this.panel.webview);
  }

  reveal(column: ViewColumn) {
    this.panel.reveal(column, true);
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
    const resourcesUri = getUri(["resources"]);

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
      <script>
        window.resourcesUri = "${resourcesUri.toString()}";
      </script>
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
    log.info("Disposing webview panel", this.type);
    panelTypeToPanel[this.type].panel = undefined;
    panelTypeToPanel[this.type].state = {};
    this.panel.dispose();
  }
}

export class QSharpViewViewPanelSerializer implements WebviewPanelSerializer {
  async deserializeWebviewPanel(panel: WebviewPanel, state: any) {
    log.info("Deserializing webview panel", state);

    const panelType: PanelType = state?.viewType;

    if (
      panelType !== "estimates" &&
      panelType !== "histogram" &&
      panelType !== "help"
    ) {
      // If it was loading when closed, that's fine
      if (panelType === "loading") {
        return;
      }
      log.error("Unknown panel type", panelType);
      return;
    }

    if (panelTypeToPanel[panelType].panel !== undefined) {
      log.error("Panel of type already exists", panelType);
      return;
    }

    panelTypeToPanel[panelType].panel = new QSharpWebViewPanel(
      panelType,
      panel,
    );
  }
}
