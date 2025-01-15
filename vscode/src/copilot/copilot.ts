// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { WorkspaceConnection } from "../azure/treeView";
import { CopilotEvent } from "../commonTypes";

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

export type QuantumChatMessage = UserMessage | AssistantMessage | ToolMessage;

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
   * JSON string with argument names and their values
   */
  arguments: string;
  /**
   * The tool call id used to match the tool call responses appropriately
   */
  id: string;
};
