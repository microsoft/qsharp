// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { log } from "qsharp";

import {
  Job,
  Target,
  WorkspaceTreeItem,
  WorkspaceTreeProvider,
} from "./workspaceTree";
import {
  getJobFiles,
  getTokenForWorkspace,
  queryWorkspace,
  queryWorkspaces,
  submitJob,
} from "./workspaceQuery";
import { sampleWorkspace } from "./sampleData";
import { QuantumUris } from "./azure";
import { getResourcePath } from "../extension";
import { getQirForActiveWindow } from "../qirGeneration";

let workspaceTreeProvider: WorkspaceTreeProvider;

export function setupWorkspaces(context: vscode.ExtensionContext) {
  workspaceTreeProvider = new WorkspaceTreeProvider();
  const workspaceTree = vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  workspaceTree.onDidChangeSelection((evt) => {
    if (evt.selection.length) {
      log.debug("TreeView selection changed to ", evt.selection[0].label);
    }
  });

  vscode.commands.registerCommand(
    "quantum-target-submit",
    async (arg: WorkspaceTreeItem) => {
      const target = arg.itemData as Target;
      let qirFilePath: vscode.Uri;

      const qir = await getQirForActiveWindow();
      if (!qir) return;

      // x-hardware-target should be set to rigetti or quantinuum
      let providerId = "rigetti";

      if (target.id.startsWith("quantinuum")) {
        providerId = "quantinuum";
      } else if (target.id.startsWith("rigetti")) {
        providerId = "rigetti";
      } else {
        return;
      }

      // TODO(billti) wrap in try/catch and log error
      // Convert the ll format qir to bitcode
      const bitcodeRequest = await fetch(
        "https://qsx-proxy.azurewebsites.net/api/compile",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/octet-stream",
            "x-hardware-target": providerId,
          },
          body: qir,
        }
      );

      if (!bitcodeRequest.ok) {
        // TODO log error and exit
      }
      const bitcode = new Uint8Array(await bitcodeRequest.arrayBuffer());

      const token = await getTokenForWorkspace(arg.workspace);
      const quantumUris = new QuantumUris(
        arg.workspace.endpointUri,
        arg.workspace.id
      );

      await submitJob(token, quantumUris, bitcode, providerId, target.id);
      setTimeout(async () => {
        await queryWorkspace(arg.workspace);
        workspaceTreeProvider.updateWorkspace(arg.workspace);
      }, 1000);
    }
  );

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
      workspaceTreeProvider.refresh();
    }
  });

  vscode.commands.registerCommand("extension.qsharp.patSignin", async () => {
    const _token = await vscode.window.showInputBox({
      title: "Enter the workspace access token",
    });
    workspaceTreeProvider.updateWorkspace(sampleWorkspace);
    workspaceTreeProvider.refresh();
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
