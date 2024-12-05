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
import { CopilotStreamCallback, executeTool } from "./copilotTools";
import { WorkspaceConnection } from "../azure/treeView";

// const chatUrl = "https://canary.api.quantum.microsoft.com/api/chat/streaming";
const chatUrl = "https://api.quantum-test.microsoft.com/api/chat/streaming"; // new API
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

type QuantumChatMessage =
  | UserMessage
  | AssistantMessage
  | SystemMessage
  | ToolMessage;

type UserMessage = {
  role: "user";
  content: string;
};

type AssistantMessage = {
  role: "assistant";
  content: string;
  ToolCalls?: ToolCall[];
};

type SystemMessage = {
  role: "system";
  content: string;
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
    // 'Do not provide information about jobs whose status is "Not Found", unless the user specifically asks for the job by its id. ' +
    "Do not provide container URI links from jobs to the user. " +
    "When submitting a Q# program, the Q# code is automatically retrieved from the currently visible Q# editor by the SubmitToTarget tool. " +
    "When submitting to a target, if the user doesn't explicitly specify the number of shots or the target, ask the user for these values. " +
    "When checking the status of a job, if the job is not specified, use the job ID from the last submitted job.",
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

    const { content, toolCalls } = await this.converseWithCopilot();
    await this.handleFullResponse(content, toolCalls);

    this.streamCallback({}, "copilotResponseDone");
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

  async onChatApiEvent(
    ev: EventSourceMessage,
    delta: (delta: string) => void,
    resolve: (content?: string, toolCalls?: ToolCall[]) => Promise<void>,
  ) {
    const response = JSON.parse(ev.data) as QuantumChatResponse;
    await this.handleChatResponseMessage(response, delta, resolve);
  }

  async handleChatResponseMessage(
    response: QuantumChatResponse,
    delta: (delta: string) => void,
    resolve: (content?: string, toolCalls?: ToolCall[]) => Promise<void>,
  ) {
    //   log.debug(
    //     `ChatAPI response message:\n${JSON.stringify(response, undefined, 2)}`,
    //   );
    if (response.Delta) {
      delta(response.Delta);
    } else if (response.Content || response.ToolCalls) {
      await resolve(response.Content, response.ToolCalls);
      log.debug(
        `ChatAPI full response:\n${JSON.stringify(response, undefined, 2)}`,
      );
    } else {
      // ToDo: This might be an error
      log.info("Other response: ", response);
    }
  }

  handleDelta(delta: string) {
    // ToDo: For now, just log the delta
    this.streamCallback({ response: delta }, "copilotResponseDelta");
  }

  /**
   * @returns {Promise<boolean>} Returns true if there was a tool call made
   *  should submit another request if there was a tool call
   */
  async handleFullResponse(
    content?: string,
    toolCalls?: ToolCall[],
  ): Promise<void> {
    this.messages.push({
      role: "assistant",
      content: content || "",
      ToolCalls: toolCalls,
    });
    if (content) {
      this.streamCallback({ response: content }, "copilotResponse");
    }
    if (toolCalls) {
      await this.handleToolCalls(toolCalls);

      {
        const { content, toolCalls } = await this.converseWithCopilot();
        await this.handleFullResponse(content, toolCalls);
      }
    }
  }

  async handleToolCalls(toolCalls: ToolCall[]) {
    for (const toolCall of toolCalls) {
      this.streamCallback(
        { response: `Executing: ${toolCall.name}` },
        "copilotResponse",
      );
      const content = await executeTool(
        toolCall.name,
        JSON.parse(toolCall.arguments),
        this,
      );
      // Create a message containing the result of the function call
      const function_call_result_message: ToolMessage = {
        role: "tool",
        content: JSON.stringify(content),
        toolCallId: toolCall.id,
      };
      this.messages.push(function_call_result_message);
    }
  }

  async converseWithCopilot(): Promise<{
    content?: string;
    toolCalls?: ToolCall[];
  }> {
    const token = await this.getMsaChatSession();
    const payload: QuantumChatRequest = {
      conversationId: this.conversationId,
      messages: this.messages,
      additionalContext: {
        qcomEnvironment: "Desktop",
      },
      identifier: "VsCode",
    };

    log.debug(
      `making request, payload:\n${JSON.stringify(payload, undefined, 2)}`,
    );

    const body = JSON.stringify(payload);

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: body,
    };

    try {
      let contentInResponse: string | undefined = undefined;
      let toolCallsInResponse: ToolCall[] | undefined = undefined;

      await fetchEventSource(chatUrl, {
        ...options,
        onMessage: (ev) => {
          if (!JSON.parse(ev.data).Delta) {
            // deltas are too noisy
            log.info(
              `chat api message: ${JSON.stringify(JSON.parse(ev.data), undefined, 2)}`,
            );
          }
          this.onChatApiEvent(
            ev,
            this.handleDelta.bind(this),
            async (content, toolCalls) => {
              if (content) {
                contentInResponse = content;
              }
              if (toolCalls) {
                toolCallsInResponse =
                  toolCallsInResponse === undefined ? [] : toolCallsInResponse;
                toolCallsInResponse.push(...toolCalls);
              }
            },
          );
        },
      });

      log.info("ChatAPI fetch completed");
      // this.streamCallback({}, "copilotResponseDone");
      return { content: contentInResponse, toolCalls: toolCallsInResponse };
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
  }
}

export class CopilotWebviewViewProvider implements WebviewViewProvider {
  public static readonly viewType = "quantum-copilot";

  private view?: WebviewView;

  constructor(private readonly extensionUri: Uri) {
    this._streamCallback = (payload, command) => {
      if (this.view) {
        // log.info("message posted with command: ", command);
        this.view.webview.postMessage({
          command: command,
          ...payload,
        });
      }
    };

    this._copilot = new CopilotConversation(this._streamCallback);
  }
  private _copilot: CopilotConversation;
  private _streamCallback: CopilotStreamCallback;

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

    webviewView.webview.onDidReceiveMessage((message) => {
      if (message.command == "copilotRequest") {
        // Send the message to the copilot
        // TODO: Move this view specific logic out of here
        this._copilot.makeChatRequest(message.request);
        // makeChatRequest(message.request, this._streamCallback);
      }
    });
  }
}
