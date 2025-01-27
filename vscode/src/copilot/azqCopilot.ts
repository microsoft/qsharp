// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getRandomGuid } from "../utils";
import { log } from "qsharp-lang";
import { getAuthSession, scopes } from "../azure/auth";
import { EventSourceMessage, fetchEventSource } from "../fetch";
import { AuthenticationSession } from "vscode";
import { ConversationState, Copilot, ICopilot, ToolCall } from "./copilot";
import { QuantumChatMessage } from "../commonTypes";

const chatUrlTest = "https://api.quantum-test.microsoft.com/api/chat/streaming";
const chatUrlLocal = "https://localhost:7044/api/chat/streaming";
const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

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

export class AzureQuantumCopilot extends Copilot implements ICopilot {
  conversationId: string;
  msaChatSession?: AuthenticationSession;

  constructor(
    private env: "local" | "test",
    conversationState: ConversationState,
  ) {
    super(conversationState);
    this.conversationId = getRandomGuid();
    log.debug("Starting copilot chat request flow");
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

  getChatUrl(): string {
    return this.env === "local" ? chatUrlLocal : chatUrlTest;
  }

  override async converseWithCopilot(): Promise<{
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

      await fetchEventSource(this.getChatUrl(), {
        ...options,
        onMessage: (ev) => {
          if (!JSON.parse(ev.data).Delta) {
            log.debug(
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
