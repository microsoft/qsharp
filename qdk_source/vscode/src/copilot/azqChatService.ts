// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { AuthenticationSession } from "vscode";
import { getAuthSession, scopes } from "../azure/auth";
import { QuantumChatMessage, ToolCall } from "./shared";
import { fetchEventSource } from "../fetch";
import { getRandomGuid } from "../utils";
import { IChatService } from "./copilot";
import { AzureQuantumServiceConfig } from "./config";

const chatApp = "652066ed-7ea8-4625-a1e9-5bac6600bf06";

/**
 * Implements interaction with the Azure Quantum chat api.
 */
export class AzureQuantumChatBackend implements IChatService {
  private conversationId: string = getRandomGuid();
  private msaChatSession?: AuthenticationSession;
  private chatEndpointUrl: string;
  private chatEndpointUrlHash: string | undefined;
  private msaAuth: boolean;

  constructor(config: AzureQuantumServiceConfig) {
    this.chatEndpointUrl = config.chatEndpointUrl;
    this.msaAuth = config.msaAuth;
  }

  async getAnonymizedEnpdoint(): Promise<string> {
    // Don't send service URL directly since that may be configured by the user.
    if (this.chatEndpointUrlHash) {
      return this.chatEndpointUrlHash;
    }
    const hashBuffer = await crypto.subtle.digest(
      "SHA-256",
      new TextEncoder().encode(this.chatEndpointUrl),
    );
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    this.chatEndpointUrlHash = hashArray
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return this.chatEndpointUrlHash;
  }

  async requestChatCompletion(
    messages: QuantumChatMessage[],
    handleResponseDelta: (delta: string) => void,
  ): Promise<{
    content?: string;
    toolCalls?: ToolCall[];
  }> {
    const authHeader: Record<string, string> = this.msaAuth
      ? {
          Authorization: `Bearer ${await this.getMsaChatSession()}`,
        }
      : {};

    const payload: QuantumChatRequest = {
      conversationId: this.conversationId,
      messages,
      additionalContext: {
        qcomEnvironment: "Desktop",
      },
      identifier: "VsCode",
    };

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...authHeader,
      },
      body: JSON.stringify(payload),
    };

    try {
      let content: string | undefined = undefined;
      let toolCalls: ToolCall[] | undefined = undefined;

      await fetchEventSource(this.chatEndpointUrl, {
        ...options,
        onMessage: (ev) => {
          if (!JSON.parse(ev.data).Delta) {
            log.debug(
              `chat api message: ${JSON.stringify(JSON.parse(ev.data), undefined, 2)}`,
            );
          }

          const message = JSON.parse(ev.data) as QuantumChatResponse;

          if (message.Delta) {
            // message content delta
            handleResponseDelta(message.Delta);
          } else if (message.Content || message.ToolCalls) {
            if (message.Content) {
              // full message content
              content = message.Content;
            }
            if (message.ToolCalls) {
              // one or more tool calls
              toolCalls = toolCalls === undefined ? [] : toolCalls;
              toolCalls.push(...message.ToolCalls);
            }
          } else {
            log.error("Received unexpected message: ", message);
          }
        },
      });

      return { content, toolCalls };
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
  }

  private async getMsaChatSession(): Promise<string> {
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
}

/**
 * Service response schema.
 */
type QuantumChatResponse = {
  /**
   * A GUID to identify the conversation.
   * Stable across multiple chat requests in the same session.
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

/**
 * Service request schema.
 */
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
