// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getRandomGuid } from "../utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "../azure/auth";
import { EventSourceMessage, fetchEventSource } from "../fetch";
import { AuthenticationSession } from "vscode";
import { executeTool } from "./copilotTools";
import { WorkspaceConnection } from "../azure/treeView";
import { CopilotEventHandler, ICopilot } from "./copilot";

// const chatUrl = "https://api.quantum-test.microsoft.com/api/chat/streaming";
const chatUrl = "https://localhost:7044/api/chat/streaming";
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

export type ToolCall = {
  /**
   * The name of the function to call
   */
  name: string;
  /**
   * Dictionary of the argument names and their values
   */
  arguments: any;
  /**
   * The tool call id used to match the tool call responses appropriately
   */
  id: string;
};

type QuantumChatResponse = {
  /**
   * The unique id for the conversation. Should be a GUID.
   */
  ConversationId: string;
  /**
   * The role of the author of this message, e.g. "assistant"
   */
  Role?: string;
  /**
   * The full content of the message.
   */
  Content?: string;
  /**
   * The tool calls that should be made as a result of this message.
   */
  ToolCalls?: ToolCall[];
  /**
   * The delta containing the fields that have changed on the Message.
   */
  Delta?: string;
  /**
   * The reason the model stopped generating tokens, e.g. "stop"|"content_filter"|"length"|null
   */
  FinishReason?: string;
  EmbeddedData: any;
  Created: string; // e.g. "2021-09-29T17:00:00.000Z"
};

type QuantumChatRequest = {
  /**
   * The unique id for the conversation. Should be a GUID.
   */
  conversationId: string;
  /**
   * The entire conversation so far.
   */
  messages: QuantumChatMessage[];
  additionalContext: any;
  identifier: string;
};

export type ConversationState = {
  activeWorkspace?: WorkspaceConnection;
  sendMessage: CopilotEventHandler;
};

export class AzureQuantumCopilot implements ICopilot {
  conversationId: string;
  messages: QuantumChatMessage[] = [];
  conversationState: ConversationState;
  msaChatSession?: AuthenticationSession;

  constructor(sendMessage: CopilotEventHandler) {
    this.conversationId = getRandomGuid();
    this.conversationState = { sendMessage };
    log.debug("Starting copilot chat request flow");
  }

  async converse(question: string): Promise<void> {
    this.messages.push({
      role: "user",
      content: question,
    });

    const { content, toolCalls } = await this.converseWithCopilot();
    await this.handleFullResponse(content, toolCalls);

    this.conversationState.sendMessage({
      kind: "copilotResponseDone",
      payload: { history: this.messages },
    });
  }

  async getMsaChatSession(): Promise<string> {
    if (!this.msaChatSession) {
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
    this.conversationState.sendMessage({
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
      this.conversationState.sendMessage({
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
      this.conversationState.sendMessage({
        kind: "copilotToolCall",
        payload: { toolName: toolCall.name },
      });
      const content = await executeTool(
        toolCall.name,
        JSON.parse(toolCall.arguments),
        this.conversationState,
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
    // const token = await this.getMsaChatSession();
    const token = "XXXX";
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
      return { content: contentInResponse, toolCalls: toolCallsInResponse };
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
  }
}
