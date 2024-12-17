import {
  WebviewViewProvider,
  WebviewView,
  Uri,
  WebviewViewResolveContext,
  CancellationToken,
} from "vscode";
import { OpenAICopilot } from "../copilot2";
import { AzureQuantumCopilot } from "./azqCopilot";
import { ICopilot, CopilotEventHandler } from "./copilot";

export class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {
    this._streamCallback = ({ payload, kind }) => {
      if (this.view) {
        // log.info("message posted with command: ", command);
        this.view.webview.postMessage({
          command: kind,
          ...payload,
        });
      }
    };

    this._copilot = new AzureQuantumCopilot(this._streamCallback);
  }
  private _copilot: ICopilot;
  private _streamCallback: CopilotEventHandler;

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

    const copilotJs = getUri(["out", "copilotExtensionPane", "copilot.js"]);
    const copilotCss = getUri(["out", "copilotExtensionPane", "copilot.css"]);
    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const githubCss = getUri(["out", "katex", "github-markdown-light.css"]);

    webviewView.webview.html = `<!DOCTYPE html>
    <html lang="en">
    <head>
    <link rel="stylesheet" href="${githubCss}" />
    <link rel="stylesheet" href="${katexCss}" />
    <link rel="stylesheet" href="${copilotCss}" />
    </head>
    <body class="markdown-body" data-theme="light">
    <script src="${copilotJs}"></script>
    </body>
    </html>`;

    webviewView.webview.onDidReceiveMessage(
      this.handleMessageFromWebview.bind(this),
    );
  }

  handleMessageFromWebview(message: MessageToCopilot) {
    if (message.command == "copilotRequest") {
      this._copilot.converse(message.request);
    } else if (message.command == "resetCopilot") {
      if (message.request === "AzureQuantum") {
        this._copilot = new AzureQuantumCopilot(this._streamCallback);
      } else if (message.request === "OpenAI") {
        this._copilot = new OpenAICopilot(this._streamCallback);
      }
    }
  }
}

export type MessageToCopilot =
  | {
      command: "copilotRequest";
      request: string;
    }
  | {
      command: "resetCopilot";
      request: "AzureQuantum" | "OpenAI";
    };
