// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import vscode from "vscode";
import { getInitialWorkspace } from "./copilotTools";
import { ConversationState } from "./azqCopilot";

export async function handleConnectToWorkspace(
  conversationState: ConversationState,
): Promise<{ result: string }> {
  try {
    await vscode.commands.executeCommand("qsharp-vscode.workspacesAdd");
  } catch {
    return {
      result: `An error occurred while trying to connect to an Azure Quantum Workspace.`,
    };
  }
  const workspace = await getInitialWorkspace();
  conversationState.activeWorkspace = workspace;

  return {
    result: "Connected to Azure Quantum Workspace: `" + workspace.name + "`",
  };
}
