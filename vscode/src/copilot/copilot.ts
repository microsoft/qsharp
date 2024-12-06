// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/**
 * Messages that get sent across the webview <-> extension boundary
 * for the copilot feature.
 */
type MessageFromCopilot =
  | { kind: "copilotResponseDelta"; payload: { response: string } }
  | { kind: "copilotResponse"; payload: { response: string } }
  | { kind: "copilotToolCall"; payload: { toolName: string } }
  | { kind: "copilotResponseDone"; payload: undefined }
  | {
      kind: "copilotResponseHistogram";
      payload: {
        buckets: [string, number][];
        shotCount: number;
      };
    };

export type CopilotMessageHandler = (msg: MessageFromCopilot) => void;

export interface ICopilotFactory {
  createCopilot(callback: CopilotMessageHandler): Promise<ICopilot>;
}

export interface ICopilot {
  converse(question: string): Promise<void>;
}
