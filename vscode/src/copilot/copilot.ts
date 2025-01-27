// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { WorkspaceConnection } from "../azure/treeView";
import { CopilotEvent, QuantumChatMessage } from "../commonTypes";
import { executeTool } from "./copilotTools";

export type CopilotEventHandler = (event: CopilotEvent) => void;

export interface ICopilotFactory {
  createCopilot(callback: CopilotEventHandler): Promise<ICopilot>;
}

export interface ICopilot {
  converse(question: string): Promise<void>;
}

export type ConversationState = {
  activeWorkspace?: WorkspaceConnection;
  messages: QuantumChatMessage[];
  sendMessage: CopilotEventHandler;
};

export type ToolCall = {
  /**
   * The name of the function to call
   */
  name: string;
  /**
   * JSON string with argument names and their values
   */
  arguments: string;
  /**
   * The tool call id used to match the tool call responses appropriately
   */
  id: string;
};

export class Copilot {
  protected messages: QuantumChatMessage[];
  constructor(protected conversationState: ConversationState) {
    this.messages = this.conversationState.messages;
  }

  async converse(question: string): Promise<void> {
    this.messages.push({ role: "user", content: question });

    const { content, toolCalls } = await this.converseWithCopilot();
    await this.handleFullResponse(content, toolCalls);

    this.conversationState.sendMessage({
      kind: "copilotResponseDone",
      payload: { history: this.messages },
    });
  }

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
      // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
      let cleanedResponse = content;
      cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
      cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");

      this.conversationState.sendMessage({
        kind: "copilotResponse",
        payload: { response: cleanedResponse, history: this.messages },
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
      const args = JSON.parse(toolCall.arguments);
      const result = await executeTool(
        toolCall.name,
        args,
        this.conversationState,
      );

      // Create a message containing the result of the function call
      const toolMessage: QuantumChatMessage = {
        role: "tool",
        content: JSON.stringify(result),
        toolCallId: toolCall.id,
      };
      this.messages.push(toolMessage);
      this.conversationState.sendMessage({
        kind: "copilotToolCallDone",
        payload: {
          toolName: toolCall.name,
          args,
          result,
          history: this.messages,
        },
      });
    }
  }

  protected async converseWithCopilot(): Promise<{
    content?: string;
    toolCalls?: ToolCall[];
  }> {
    throw new Error("Method not implemented.");
  }
}
