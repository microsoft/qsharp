// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// These are types that are common to
// webviews and the vscode extension

export type MessageToCopilot =
  | {
      command: "copilotRequest";
      request: string;
    }
  | {
      command: "resetCopilot";
      request: "AzureQuantumTest" | "AzureQuantumLocal" | "OpenAI";
    }
  | {
      command: "retryRequest";
      service: "AzureQuantumTest" | "AzureQuantumLocal" | "OpenAI";
    };

export type ServiceTypes = "AzureQuantumLocal" | "AzureQuantumTest" | "OpenAI";

/**
 * Events that get sent across the webview <-> extension boundary
 * for the copilot feature.
 */
export type CopilotEvent =
  | { kind: "copilotResponseDelta"; payload: { response: string } }
  | { kind: "copilotResponse"; payload: { response: string } }
  | { kind: "copilotToolCall"; payload: { toolName: string } }
  | {
      kind: "copilotToolCallDone";
      payload: {
        toolName: string;
        args: object;
        result: object;
        history: object[];
      };
    }
  | { kind: "copilotResponseDone"; payload: { history: object[] } }
  | {
      kind: "copilotResponseHistogram";
      payload: {
        response: string;
      };
    };
