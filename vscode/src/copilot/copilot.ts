// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getRandomGuid } from "../utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "../azure/auth";
import { EventSourceMessage, fetchEventSource } from "../fetch";
import {
  AuthenticationSession,
  CancellationToken,
  Uri,
  WebviewView,
  WebviewViewProvider,
  WebviewViewResolveContext,
} from "vscode";
import { CopilotStreamCallback, ToolCallSwitch } from "./copilotTools";
import { WorkspaceConnection } from "../azure/treeView";

// const chatUrl = "https://canary.api.quantum.microsoft.com/api/chat/streaming";
const chatUrl = "https://api.quantum-test.microsoft.com/api/chat/streaming"; // new API
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

type QuantumChatMessage = UserMessage | AssistantMessage | ToolMessage;

type UserMessage = {
  role: "user";
  content: string;
};

type AssistantMessage = {
  role: "assistant";
  content: string;
  ToolCalls?: ToolCall[];
};

type ToolMessage = {
  role: "tool";
  content: string;
  toolCallId?: string;
};

type ToolCall = {
  name: string; // The name of the function to call
  arguments: any; // Dictionary of the argument names and their values
  id: string; // The tool call id used to match the toll call responses appropriately
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

const systemMessage: AssistantMessage = {
  role: "assistant",
  content:
    "You are a helpful customer support assistant. Use the supplied tools to assist the user. " +
    'Do not provide information about jobs whose status is "Not Found", unless the user specifically asks for the job by it\'s id. ' +
    "Do not provide container URI links from jobs to the user. ",
};

type QuantumChatRequest = {
  conversationId: string; // GUID
  messages: QuantumChatMessage[];
  additionalContext: any;
  identifier: string;
};

export class CopilotConversation {
  conversationId: string;
  messages: QuantumChatMessage[] = [];
  active_workspace?: WorkspaceConnection;
  streamCallback: CopilotStreamCallback;
  _msaChatSession?: AuthenticationSession;

  constructor(streamCallback: CopilotStreamCallback) {
    this.conversationId = getRandomGuid();
    this.messages.push(systemMessage);
    this.streamCallback = streamCallback;
    log.debug("Starting copilot chat request flow");
  }

  async makeChatRequest(question: string) {
    this.messages.push({
      role: "user",
      content: question,
    });

    await this.converseWithCopilot();
  }

  async getMsaChatSession(): Promise<string> {
    if (!this._msaChatSession) {
      log.info("new token");
      this._msaChatSession = await getAuthSession(
        [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
        getRandomGuid(),
      );
      if (!this._msaChatSession) {
        throw Error("Failed to get MSA chat token");
      }
    }
    return this._msaChatSession.accessToken;
  }

  async onMessage(ev: EventSourceMessage) {
    const messageReceived: QuantumChatResponse = JSON.parse(ev.data);
    await this.handleResponse(messageReceived);
  }

  async handleResponse(response: QuantumChatResponse) {
    if (response.Delta) {
      // ToDo: For now, just log the delta
      // this.streamCallback({ response: response.Delta }, "copilotResponse");
      // log.info("Delta: ", response.Delta);
    } else if (response.Content || response.ToolCalls) {
      this.messages.push({
        role: "assistant",
        content: response.Content || "",
        ToolCalls: response.ToolCalls,
      });
      if (response.Content) {
        this.streamCallback({ response: response.Content }, "copilotResponse");
      }
      if (response.ToolCalls) {
        log.info("Tools Call message: ", response);
        await this.handleToolCalls(response);
      }
    } else {
      // ToDo: This might be an error
      log.info("Other response: ", response);
    }
  }

  async handleToolCalls(response: QuantumChatResponse) {
    if (response.ToolCalls) {
      for (const toolCall of response.ToolCalls) {
        const content = await this.handleSingleToolCall(toolCall);
        // Create a message containing the result of the function call
        const function_call_result_message: ToolMessage = {
          role: "tool",
          content: JSON.stringify(content),
          toolCallId: toolCall.id,
        };
        this.messages.push(function_call_result_message);
      }

      await this.converseWithCopilot();
    }
  }

  async handleSingleToolCall(toolCall: ToolCall) {
    const args = JSON.parse(toolCall.arguments);
    return ToolCallSwitch(toolCall.name, args, this);
  }

  async converseWithCopilot() {
    const token = await this.getMsaChatSession();
    const payload: QuantumChatRequest = {
      conversationId: this.conversationId,
      messages: this.messages,
      additionalContext: {
        qcomEnvironment: "Desktop",
      },
      identifier: "VsCode",
    };

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(payload),
    };

    // log.info("About to call ChatAPI with payload: ", options);
    try {
      const onMessage = this.onMessage.bind(this);
      await fetchEventSource(chatUrl, {
        ...options,
        onMessage,
      });

      log.info("ChatAPI fetch completed");
      this.streamCallback({}, "copilotResponseDone");
      return {};
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
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
