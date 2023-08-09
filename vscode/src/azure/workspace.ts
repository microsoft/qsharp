// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { log } from "qsharp";

import { Job, WorkspaceTreeItem, WorkspaceTreeProvider } from "./workspaceTree";
import {
  getJobFiles,
  getTokenForWorkspace,
  queryWorkspaces,
} from "./workspaceQuery";
import { sampleResult, sampleWorkspace } from "./sampleData";
import { QuantumUris } from "./azure";

let workspaceTreeProvider: WorkspaceTreeProvider;

export function setupWorkspaces(context: vscode.ExtensionContext) {
  workspaceTreeProvider = new WorkspaceTreeProvider();
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
    if (!method) return;
    if (method === tokenPrompt) {
      vscode.commands.executeCommand("extension.qsharp.patSignin");
    } else {
      vscode.commands.executeCommand("extension.qsharp.aadSignin");
    }
  });

  vscode.commands.registerCommand("extension.qsharp.aadSignin", async () => {
    const workspace = await queryWorkspaces();
    if (workspace) {
      workspaceTreeProvider.updateWorkspace(workspace);
    }
  });

  vscode.commands.registerCommand("extension.qsharp.patSignin", async () => {
    const _token = await vscode.window.showInputBox({
      title: "Enter the workspace access token",
    });
    workspaceTreeProvider.updateWorkspace(sampleWorkspace);
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
  vscode.commands.registerCommand(
    "quantum-result-download",
    async (arg: WorkspaceTreeItem) => {
      const job = arg.itemData as Job;

      if (!job.outputDataUri) return;

      const fileUri = vscode.Uri.parse(job.outputDataUri);
      const [_, container, blob] = fileUri.path.split("/");

      const token = await getTokenForWorkspace(arg.workspace);
      const quantumUris = new QuantumUris(
        arg.workspace.endpointUri,
        arg.workspace.id
      );

      const file = await getJobFiles(container, blob, token, quantumUris);
      if (file) {
        const doc = await vscode.workspace.openTextDocument({
          content: file,
          language: "plaintext",
        });
        vscode.window.showTextDocument(doc);
      }
    }
  );
}
