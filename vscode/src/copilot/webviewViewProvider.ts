import {
  WebviewViewProvider,
  WebviewView,
  Uri,
  WebviewViewResolveContext,
  CancellationToken,
} from "vscode";
import { OpenAICopilot } from "./openAiCopilot";
import { AzureQuantumCopilot } from "./azqCopilot";
import {
  ICopilot,
  CopilotEventHandler,
  ConversationState,
  QuantumChatMessage,
} from "./copilot";
import { MessageToCopilot, ServiceTypes } from "../commonTypes";

export class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {
    this._streamCallback = ({ payload, kind }) => {
      if (this.view) {
        // log.info("message posted with command: ", command);
        this.view.webview.postMessage({
          kind,
          payload,
        });
      }
    };

    this._conversationState = {
      messages: [],
      sendMessage: this._streamCallback,
    };

    this._copilot = new AzureQuantumCopilot("local", this._conversationState);
    this._serviceType = "AzureQuantumLocal";
  }

  private _copilot: ICopilot;
  private _serviceType: ServiceTypes;
  private _conversationState: ConversationState;
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

    const copilotJs = getUri([
      "out",
      "webviews",
      "copilotExtensionPane",
      "copilot.js",
    ]);
    const copilotCss = getUri([
      "out",
      "webviews",
      "copilotExtensionPane",
      "copilot.css",
    ]);
    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const githubCss = getUri(["out", "katex", "github-markdown-light.css"]);

    webviewView.webview.html = `<!DOCTYPE html>
    <html lang="en">
    <head>
    <link rel="stylesheet" href="${githubCss}" />
    <link rel="stylesheet" href="${katexCss}" />
    <link rel="stylesheet" href="${copilotCss}" />
    </head>
    <body>
    <script src="${copilotJs}"></script>
    </body>
    </html>`;

    webviewView.webview.onDidReceiveMessage(
      this.handleMessageFromWebview.bind(this),
    );
  }

  handleMessageFromWebview(message: MessageToCopilot) {
    switch (message.command) {
      case "copilotRequest": {
        this._copilot.converse(message.request);
        break;
      }
      case "resetCopilot": {
        this._serviceType = message.request;
        // fresh conversation state
        this._conversationState = {
          messages: [],
          sendMessage: this._streamCallback,
        };
        switch (message.request) {
          case "AzureQuantumLocal":
            this._copilot = new AzureQuantumCopilot(
              "local",
              this._conversationState,
            );
            break;
          case "AzureQuantumTest":
            this._copilot = new AzureQuantumCopilot(
              "test",
              this._conversationState,
            );
            break;
          case "OpenAI":
            this._copilot = new OpenAICopilot(this._conversationState);
            break;
        }
        break;
      }
      case "retryRequest": {
        // roll back until there is a "user" message
        let lastMessage: QuantumChatMessage | undefined;
        while (this._conversationState.messages.length > 0) {
          lastMessage = this._conversationState.messages.pop();
          if (lastMessage?.role === "user") {
            break;
          }
        }

        if (!lastMessage) {
          // no user message found
          throw new Error("I did not account for this");
        }

        if (this._serviceType !== message.service) {
          this._serviceType = message.service;
          switch (message.service) {
            case "AzureQuantumLocal":
              this._copilot = new AzureQuantumCopilot(
                "local",
                this._conversationState, // keep the conversation state
              );
              break;
            case "AzureQuantumTest":
              this._copilot = new AzureQuantumCopilot(
                "test",
                this._conversationState, // keep the conversation state
              );
              break;
            case "OpenAI":
              this._copilot = new OpenAICopilot(
                this._conversationState, // keep the conversation state
              );
              break;
          }
        }

        this._copilot.converse(lastMessage.content);
        break;
      }
    }
  }
}
