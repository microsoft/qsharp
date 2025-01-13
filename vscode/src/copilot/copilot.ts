// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/**
 * Events that get sent across the webview <-> extension boundary
 * for the copilot feature.
 */
type CopilotEvent =
  | { kind: "copilotResponseDelta"; payload: { response: string } }
  | { kind: "copilotResponse"; payload: { response: string } }
  | { kind: "copilotToolCall"; payload: { toolName: string } }
  | { kind: "copilotResponseDone"; payload: { history: object[] } }
  | {
      kind: "copilotResponseHistogram";
      payload: {
        buckets: [string, number][];
        shotCount: number;
      };
    };

export type CopilotEventHandler = (event: CopilotEvent) => void;

export interface ICopilotFactory {
  createCopilot(callback: CopilotEventHandler): Promise<ICopilot>;
}

export interface ICopilot {
  converse(question: string): Promise<void>;
}
