// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */
import { getRandomGuid } from "./utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "./azure/auth";
import { fetchEventSource } from "./fetch";
import {
  CancellationToken,
  Uri,
  WebviewView,
  WebviewViewProvider,
  WebviewViewResolveContext,
} from "vscode";
import { CopilotStreamCallback } from "./copilotTools";

// const chatUrl = "https://westus3.aqa.quantum.azure.com/api/chat/streaming";
const chatUrl = "https://api.quantum-test.microsoft.com/api/chat/streaming"; // new API
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

const latexContext = String.raw`Please ensure any LaTeX code is enclosed in single or double dollar signs, e.g. $x^2$ or $$x^2$$ , and not escaped parentheses, e.g. \(x^2\).`;

type quantumChatRequest = {
  conversationId: string; // GUID
  messages: Array<{
    role: string; // e.g. "user"
    content: string;
  }>; // The actual question
  additionalContext: any;
  identifier: string;
};

type QuantumChatResponse = {
  ConversationId: string; // GUID,
  Role: string; // e.g. "assistant"
  Content?: string; // The full response
  ToolCalls?: ToolCall[];
  Delta?: string; // The next response token
  FinishReason?: string; // e.g. "stop"|"content_filter"|"length"|null,
  EmbeddedData: any;
  Created: string; // e.g. "2021-09-29T17:00:00.000Z"
};

type ToolCall = {
  name: string; // The name of the function to call
  arguments: any; // dictionary of the argument names and their values
  id: string; // the tool call id used to match the toll call responses appropriately
};

export async function makeChatRequest(
  question: string,
  streamCallback: CopilotStreamCallback,
) {
  log.debug("Starting copilot chat request flow");
  const msaChatSession = await getAuthSession(
    [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
    getRandomGuid(),
  );
  if (!msaChatSession) {
    throw Error("Failed to get MSA chat token");
  }

  await chatRequest(
    msaChatSession.accessToken,
    question,
    streamCallback,
    latexContext,
  );
}

async function chatRequest(
  token: string,
  question: string,
  streamCallback: CopilotStreamCallback,
  context?: string,
): Promise<any> {
  log.debug("Requesting response");
  const payload: quantumChatRequest = {
    conversationId: getRandomGuid(),
    messages: [
      {
        role: "user",
        content: question,
      },
    ],
    additionalContext: {
      qcomEnvironment: "Desktop",
    },
    identifier: "Quantum",
  };

  if (context) {
    payload.messages.unshift({
      role: "assistant",
      content: context,
    });
  }

  const options = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  };

  try {
    log.debug("About to call ChatAPI with payload: ", payload);
    await fetchEventSource(chatUrl, {
      ...options,
      onMessage(ev) {
        log.info("Received copilot fetch message: ", ev);
        const messageReceived: QuantumChatResponse = JSON.parse(ev.data);
        log.info("Received message: ", messageReceived);
        if (messageReceived.Delta) {
          streamCallback(
            {
              response: messageReceived.Delta,
            },
            "copilotResponse",
          );
        } else if (messageReceived.ToolCalls) {
          // ToDo
          log.info("Tools Call message: ", messageReceived);
        }
      },
    });

    log.debug("ChatAPI fetch completed");
    streamCallback({}, "copilotResponseDone");
    return Promise.resolve({});
  } catch (error) {
    log.error("ChatAPI fetch failed with error: ", error);
    throw error;
  }
}

export class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {}

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
    const copilotJs = getUri(["out", "copilot", "copilot.js"]);
    const copilotCss = getUri(["out", "copilot", "copilot.css"]);
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
  }
}
