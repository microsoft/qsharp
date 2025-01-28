// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  IOperationInfo,
  QscEventTarget,
  VSDiagnostic,
  getCompilerWorker,
  log,
} from "qsharp-lang";
import {
  ExtensionContext,
  Uri,
  ViewColumn,
  Webview,
  WebviewPanel,
  WebviewPanelSerializer,
  commands,
  window,
} from "vscode";
import { showCircuitCommand } from "./circuit";
import { clearCommandDiagnostics } from "./diagnostics";
import { showDocumentationCommand } from "./documentation";
import { getActiveProgram } from "./programConfig";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";
import { getPauliNoiseModel } from "./config";
import { qsharpExtensionId } from "./common";

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
    commands.registerCommand(`${qsharpExtensionId}.showRe`, async () => {
      clearCommandDiagnostics();
      const associationId = getRandomGuid();
      sendTelemetryEvent(
        EventType.TriggerResourceEstimation,
        { associationId },
        {},
      );
      const program = await getActiveProgram();
      if (!program.success) {
        throw new Error(program.errorMsg);
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

      let runName = await window.showInputBox({
        title: "Friendly name for run",
        value: `${program.programConfig.projectName}`,
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

      sendMessageToPanel({ panelType: "estimates" }, true, {
        calculating: true,
      });

      const estimatePanel = getOrCreatePanel("estimates");
      // Ensure the name is unique
      if (estimatePanel.state[runName] !== undefined) {
        let idx = 2;
        for (;;) {
          const newName = `${runName}-${idx}`;
          if (estimatePanel.state[newName] === undefined) {
            runName = newName;
            break;
          }
          idx++;
        }
      }
      estimatePanel.state[runName] = true;

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
        const start = performance.now();
        sendTelemetryEvent(
          EventType.ResourceEstimationStart,
          { associationId },
          {},
        );
        const estimatesStr = await worker.getEstimates(
          program.programConfig,
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
          (item) => (item.jobParams.sharedRunName = runName),
        );

        clearTimeout(compilerTimeout);

        const message = {
          calculating: false,
          estimates,
        };
        sendMessageToPanel({ panelType: "estimates" }, true, message);
      } catch (e: any) {
        // Stop the 'calculating' animation
        const message = {
          calculating: false,
          estimates: [],
        };
        sendMessageToPanel({ panelType: "estimates" }, false, message);

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
    commands.registerCommand(`${qsharpExtensionId}.showHelp`, async () => {
      const message = {};
      sendMessageToPanel({ panelType: "help" }, true, message);
    }),
  );

  context.subscriptions.push(
    commands.registerCommand(`${qsharpExtensionId}.showHistogram`, async () => {
      clearCommandDiagnostics();

      const associationId = getRandomGuid();
      sendTelemetryEvent(EventType.TriggerHistogram, { associationId }, {});
      function resultToLabel(result: string | VSDiagnostic): string {
        if (typeof result !== "string") return "ERROR";
        return result;
      }

      const program = await getActiveProgram();
      if (!program.success) {
        throw new Error(program.errorMsg);
      }

      const panelId = program.programConfig.projectName;

      // Start the worker, run the code, and send the results to the webview
      const worker = getCompilerWorker(compilerWorkerScriptPath);
      const compilerTimeout = setTimeout(() => {
        worker.terminate();
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

        sendMessageToPanel(
          { panelType: "histogram", id: panelId },
          true,
          undefined,
        );

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
            buckets: Array.from(buckets.entries()),
            shotCount: resultCount,
          };
          sendMessageToPanel(
            { panelType: "histogram", id: panelId },
            false,
            message,
          );
        });
        const start = performance.now();
        sendTelemetryEvent(EventType.HistogramStart, { associationId }, {});

        const noise = getPauliNoiseModel();
        if (noise[0] != 0 || noise[1] != 0 || noise[2] != 0) {
          sendTelemetryEvent(EventType.NoisySimulation, { associationId }, {});
        }
        await worker.runWithPauliNoise(
          program.programConfig,
          "",
          parseInt(numberOfShots),
          noise,
          evtTarget,
        );
        sendTelemetryEvent(
          EventType.HistogramEnd,
          { associationId },
          { timeToCompleteMs: performance.now() - start },
        );
        clearTimeout(compilerTimeout);
      } catch (e: any) {
        log.error("Histogram error. ", e.toString());
        throw new Error("Run failed. " + e.toString());
      } finally {
        worker.terminate();
      }
    }),
  );

  context.subscriptions.push(
    commands.registerCommand(
      `${qsharpExtensionId}.showCircuit`,
      async (operation?: IOperationInfo) => {
        await showCircuitCommand(context.extensionUri, operation);
      },
    ),
  );

  context.subscriptions.push(
    commands.registerCommand(
      `${qsharpExtensionId}.showDocumentation`,
      async () => {
        await showDocumentationCommand(context.extensionUri);
      },
    ),
  );
}

type PanelDesc = {
  title: string;
  panel: QSharpWebViewPanel;
  state: any;
};

type PanelType =
  | "histogram"
  | "estimates"
  | "help"
  | "circuit"
  | "documentation";

const panels: Record<PanelType, { [id: string]: PanelDesc }> = {
  histogram: {},
  estimates: {},
  circuit: {},
  help: {},
  documentation: {},
};

const panelTypeToTitle: Record<PanelType, string> = {
  histogram: "Q# Histogram",
  estimates: "Q# Estimates",
  circuit: "Q# Circuit",
  help: "Q# Help",
  documentation: "Q# Documentation",
};

function getPanel(type: PanelType, id?: string): PanelDesc | undefined {
  if (id) {
    return panels[type][id];
  } else {
    return panels[type][""];
  }
}

export function isPanelOpen(panelType: PanelType, id?: string): boolean {
  return getPanel(panelType, id)?.panel !== undefined;
}

function createPanel(
  type: PanelType,
  id?: string,
  webViewPanel?: WebviewPanel,
): PanelDesc {
  if (id == undefined) {
    id = "";
  }
  if (webViewPanel) {
    const title = webViewPanel.title;
    const panel = new QSharpWebViewPanel(type, webViewPanel, id);
    panels[type][id] = { title, panel, state: {} };
    return panels[type][id];
  } else {
    let title = `${panelTypeToTitle[type]}`;
    if (type == "circuit" || type == "histogram") {
      title = title + ` ${id}`;
    }
    const newPanel = window.createWebviewPanel(
      QSharpWebViewType,
      title,
      {
        viewColumn: ViewColumn.Three,
        preserveFocus: true,
      },
      {
        enableCommandUris: true,
        enableScripts: true,
        enableFindWidget: true,
        retainContextWhenHidden: true,
        // Note: If retainContextWhenHidden is false, the webview gets reloaded
        // every time you hide it by switching to another tab and then switch
        // back. While we've done the work to persist the underlying state, the
        // dynamic state of the DOM on the page - such as which detail sections are
        // expanded, whether in summary or detail view, etc. - is lost if this occurs.
      },
    );

    const panel = new QSharpWebViewPanel(type, newPanel, id);
    panels[type][id] = { title, panel, state: {} };
    return panels[type][id];
  }
}

function getOrCreatePanel(type: PanelType, id?: string): PanelDesc {
  const panel = getPanel(type, id);
  if (panel) {
    return panel;
  } else {
    return createPanel(type, id);
  }
}

export function sendMessageToPanel(
  panel: { panelType: PanelType; id?: string },
  reveal: boolean,
  message: any,
) {
  const panelRecord = getOrCreatePanel(panel.panelType, panel.id);
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
    private id: string,
  ) {
    log.info(`Creating webview panel of type ${type} and id ${id}`);
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
    const githubCss = getUri(["out", "katex", "github-markdown-dark.css"]);
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
    message.command = message.command || this.type;
    message.panelId = message.panelId || this.id;
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
    const panel = getPanel(this.type, this.id);
    if (panel) {
      panel.state = {};
      delete panels[this.type][this.id];
    }
    this.panel.dispose();
  }
}

export class QSharpViewViewPanelSerializer implements WebviewPanelSerializer {
  async deserializeWebviewPanel(panel: WebviewPanel, state: any) {
    log.info("Deserializing webview panel", state);

    const panelType: PanelType = state?.viewType;
    const id = state?.panelId;

    if (
      panelType !== "estimates" &&
      panelType !== "histogram" &&
      panelType !== "circuit" &&
      panelType !== "help" &&
      panelType != "documentation"
    ) {
      // If it was loading when closed, that's fine
      if (panelType === "loading") {
        return;
      }
      log.error("Unknown panel type", panelType);
      return;
    }

    if (getPanel(panelType, id) !== undefined) {
      log.error(`Panel of type ${panelType} and id ${id} already exists`);
      return;
    }

    createPanel(panelType, id, panel);
  }
}
