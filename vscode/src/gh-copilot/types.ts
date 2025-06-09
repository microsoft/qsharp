// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/**
 * An error thrown by a tool when the tool cannot complete its task.
 * The message will be shown to the user.
 */
export class CopilotToolError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CopilotToolError";
  }
}

/**
 * State that can be shared between tool calls in a conversation.
 */
export type ToolState = Record<string, any>;

/**
 * Histogram data for displaying results
 */
export type HistogramData = {
  buckets: [string, number][];
  shotCount?: number;
};
