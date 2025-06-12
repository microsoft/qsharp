// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { WorkspaceConnection } from "../azure/treeView";
import { HistogramData } from "./shared.js";
import { azqToolDefinitions } from "./azqTools.js";

/**
 * Executes a tool as requested by the assistant.
 * The name and args were provided in the assistant response.
 */
export async function executeTool(
  tool_name: string,
  jsonArgs: string,
  toolState: ToolState,
): Promise<ToolResult> {
  let args;
  try {
    // The service should return valid JSON for arguments,
    // but validate anyway. Different services may hallucinate syntax.
    args = JSON.parse(jsonArgs);
  } catch (e) {
    log.error("Invalid tool call arguments", jsonArgs, e);
    return { error: "Invalid argument" };
  }

  const result: any = {};

  const handler = azqToolDefinitions[tool_name];

  if (handler) {
    try {
      // Don't remove this `await`. It's necessary for the try/catch block
      // to actually handle exceptions.
      return await handler(toolState, args);
    } catch (e) {
      if (e instanceof CopilotToolError) {
        return { error: e.message };
      }
      // We'll avoid adding arbitrary error details to the conversation history
      // since they can get large and use up a lot of tokens with essentially noise.
      //
      // If you need to include the error details for a specific error, catch
      // it and rethrow it as a CopilotToolError the relevant context.
      return { error: "An error occurred." };
    }
  }

  return { result };
}

/**
 * The messages from these exceptions will be added to the conversation
 * history, so keep the messages meaningful to the copilot and/or user.
 */
export class CopilotToolError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CopilotToolError";
  }
}

/**
 * Result of a tool execution
 */
export type ToolResult =
  | {
      result: any;
      /**
       * Data that can be used to aid rendering in the client side.
       * but needs to be excluded from the conversation history that the
       * assistant sees to avoid confusion.
       */
      widgetData?: HistogramData; // histogram is the only widget type supported at the moment
    }
  | { error: string };

/**
 * State shared by the tool handlers.
 */
export type ToolState = {
  activeWorkspace?: WorkspaceConnection;
};
