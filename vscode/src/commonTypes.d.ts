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
    };
