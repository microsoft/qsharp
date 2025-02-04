// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// These are types that are common to
// webviews and the vscode extension

export type CopilotCommand =
  | {
      command: "submitUserMessage";
      request: string;
    }
  | {
      command: "restartChat";
      history: ChatElement[];
      service?: ServiceType;
    };

export type ServiceType = "AzureQuantumLocal" | "AzureQuantumTest";

/**
 * Events that get sent across the webview <-> extension boundary
 * for the copilot feature.
 */
export type CopilotUpdate =
  | {
      kind: "updateChat";
      payload: {
        history: ChatElement[];
        status: Status;
        serviceOptions: ServiceType[];
        service: ServiceType;
      };
    }
  | { kind: "updateStatus"; payload: { status: Status } }
  | {
      kind: "appendDelta";
      payload: { delta: string; status: Status };
    };

export type QuantumChatMessage = UserMessage | AssistantMessage | ToolMessage;

/**
 * A widget "message" is not a real message in the chat: it's
 * inserted into the chat by tool calls, and displayed
 * in the UI as if it's a message, but it's not included
 * in the chat history payload that gets sent to the service.
 */
export type ChatElement = QuantumChatMessage | Widget;

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
  toolCallId: string;
};

type Widget = {
  role: "widget";
  widgetData: HistogramData; // Only widget type supported at the moment
};

type HistogramData = {
  buckets: [string, number][];
  shotCount: number;
};

type Status =
  | { status: "ready" }
  | {
      status: "waitingAssistantResponse";
    }
  | {
      status: "executingTool";
      toolName: string;
    }
  | {
      status: "assistantConnectionError";
    };

type ToolCall = {
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
