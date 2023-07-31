// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp";

import { WorkspaceTreeProvider } from "./workspaceTree";
import { queryWorkspaces } from "./workspaceQuery";

let workspaceTreeProvider: WorkspaceTreeProvider;

export function setupWorkspaces(context: vscode.ExtensionContext) {
  workspaceTreeProvider = new WorkspaceTreeProvider(context);
  const workspaceTree = vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  workspaceTree.onDidChangeSelection((evt) => {
    if (evt.selection.length) {
      log.debug("TreeView selection changed to ", evt.selection[0].label);
      evt.selection[0];
    }
  });

  vscode.commands.registerCommand("quantum-target-submit", () => {
    vscode.window.showErrorMessage(
      `The current target does not support all features required by this program.
  Please resolve the error messages for the current project and try again.`,
      {
        modal: true,
        detail: "For more details, see https://aka.ms/qir-profiles",
      }
    );
  });

  vscode.commands.registerCommand("quantum-workspaces-refresh", () => {
    workspaceTreeProvider.refresh();
  });

  vscode.commands.registerCommand("quantum-workspace-getkey", async () => {
    const rawPrompt = "Get access key only";
    const pythonPrompt = "Get Python code with access key";
    const result = await vscode.window.showQuickPick(
      [rawPrompt, pythonPrompt],
      { title: "Copy workspace access key" }
    );
    if (result === rawPrompt) {
      await vscode.env.clipboard.writeText("asdlfkjwekj22343242lkdf");
    } else {
      await vscode.env.clipboard.writeText(`from azure.quantum import Workspace
workspace = new Workspace(accessKey = "q23987dasdflkjwerw235")
`);
    }
    vscode.window.showInformationMessage(
      "Workspace key copied to the clipboard"
    );
  });

  vscode.commands.registerCommand("quantum-job-cancel", async () => {
    const confirm = await vscode.window.showWarningMessage(
      "Are you sure you want to cancel the job?",
      {
        modal: true,
      },
      "Yes",
      "No"
    );
    if (confirm === "Yes") vscode.window.showInformationMessage("Job deleted");
  });

  vscode.commands.registerCommand("quantum-workspaces-add", async () => {
    const accountPrompt = "Sign-in with a Microsoft account";
    const tokenPrompt = "Connect using an access token";
    const method = await vscode.window.showQuickPick(
      [accountPrompt, tokenPrompt],
      { title: "Select authentication method" }
    );
    if (method === tokenPrompt) {
      const _token = await vscode.window.showInputBox({
        title: "Enter the workspace access token",
      });
    } else {
      // TODO: Sign-in, select tenant, etc.
      const sub = await vscode.window.showQuickPick(
        ["MSDN Subscription", "Production"],
        { title: "Select the Azure subscription" }
      );
      const workspace = await vscode.window.showQuickPick(
        ["Chemistry", "Research"],
        { title: "Select the workspace to add" }
      );
    }
  });

  vscode.commands.registerCommand("quantum-target-view", async () => {
    // TODO: Open a webview or browser window for the target
    vscode.window.showInformationMessage("All systems are go!");
  });
  vscode.commands.registerCommand("quantum-filter-results", async () => {
    // TODO: Open a webview with a histogram similar to playground
    vscode.window.showInformationMessage("TODO");
  });
  vscode.commands.registerCommand("quantum-result-histogram", async () => {
    // TODO: Open a webview with a histogram similar to playground
    vscode.window.showInformationMessage("TODO");
  });
  vscode.commands.registerCommand("quantum-result-download", async () => {
    const doc = await vscode.workspace.openTextDocument({
      content: `# Job results for provider IonQ on 2023-06-23 10::34 UTC

START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t0\t1_t0r
OUTPUT\tDOUBLE\t0.42\t2_t1d
END\t0
START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t1\t1_t0r
OUTPUT\tDOUBLE\t0.42\t2_t1d
END\t0
START
METADATA\tmetadata1_name_only
METADATA\tmetadata2_name\tmetadata2_value
METADATA\tmetadata3_name\tmetadata3_value
OUTPUT\tTUPLE\t2\t0_t
OUTPUT\tRESULT\t0\t1_t0r
OUTPUT\tDOUBLE\t0.25\t2_t1d
END\t0
`,
      language: "plaintext",
    });
    vscode.window.showTextDocument(doc);
  });

  vscode.commands.registerCommand(
    "extension.qsharp.listWorkspaces",
    queryWorkspaces
  );
}
