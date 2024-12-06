// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getRandomGuid } from "../utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "../azure/auth";
import { EventSourceMessage, fetchEventSource } from "../fetch";
import { AuthenticationSession } from "vscode";
import { executeTool } from "./copilotTools";
import { WorkspaceConnection } from "../azure/treeView";
import { CopilotMessageHandler, ICopilot } from "./copilot";

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

export class AzureQuantumCopilot implements ICopilot {
  conversationId: string;
  messages: QuantumChatMessage[] = [];
  activeWorkspace?: WorkspaceConnection;
  sendMessage: CopilotMessageHandler;
  msaChatSession?: AuthenticationSession;

  constructor(sendMessage: CopilotMessageHandler) {
    this.conversationId = getRandomGuid();
    this.messages.push(systemMessage);
    this.sendMessage = sendMessage;
    log.debug("Starting copilot chat request flow");
  }

  async converse(question: string): Promise<void> {
    this.messages.push({
      role: "user",
      content: question,
    });

    const { content, toolCalls } = await this.converseWithCopilot();
    await this.handleFullResponse(content, toolCalls);

    this.sendMessage({ kind: "copilotResponseDone", payload: undefined });
  }

  async getMsaChatSession(): Promise<string> {
    if (!this.msaChatSession) {
      log.info("new token");
      this.msaChatSession = await getAuthSession(
        [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
        getRandomGuid(),
      );
      if (!this.msaChatSession) {
        throw Error("Failed to get MSA chat token");
      }
    }
    return this.msaChatSession.accessToken;
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
    this.sendMessage({
      kind: "copilotResponseDelta",
      payload: { response: delta },
    });
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
      this.sendMessage({
        kind: "copilotResponse",
        payload: { response: content },
      });
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
      this.sendMessage({
        kind: "copilotToolCall",
        payload: { toolName: toolCall.name },
      });
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
