// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import vscode from "vscode";

export async function handleConnectToWorkspace(): Promise<{ result: string }> {
  vscode.commands.executeCommand("qsharp-vscode.workspacesAdd");
  return {
    result:
      "Please follow the prompts to connect to an existing Azure Quantum Workspace.",
  };
}
