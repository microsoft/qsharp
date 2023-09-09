// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from "vscode";
import { log } from "qsharp-lang";

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
import { QuantumUris } from "./azure";
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

      const compilerService: string | undefined = vscode.workspace
        .getConfiguration("Q#")
        .get("compilerService"); // e.g. in settings.json: "Q#.compilerService": "https://qsx-proxy.azurewebsites.net/api/compile"

      const qir = await getQirForActiveWindow();
      if (!qir) return;

      // x-hardware-target should be set to rigetti or quantinuum
      let providerId = "rigetti";

      if (target.id.startsWith("quantinuum")) {
        providerId = "quantinuum";
      } else if (target.id.startsWith("rigetti")) {
        providerId = "rigetti";
      } else if (target.id.startsWith("ionq")) {
        providerId = "ionq";
      } else {
        vscode.window.showErrorMessage(
          "Unsupported provider for QIR jobs: " + target.id
        );
        return;
      }

      let payload: Uint8Array | string = qir;

      if (compilerService) {
        try {
          log.info("Using compiler service at " + compilerService);
          const bitcodeRequest = await fetch(compilerService, {
            method: "POST",
            headers: {
              "Content-Type": "application/octet-stream",
              "x-hardware-target": providerId,
            },
            body: qir,
          });

          if (!bitcodeRequest.ok) {
            log.error("Failed to compile to QIR bitcode", bitcodeRequest);
            vscode.window.showErrorMessage("Failed to compile to QIR bitcode");
            return;
          }
          payload = new Uint8Array(await bitcodeRequest.arrayBuffer());
        } catch (e) {
          log.error("Failed to compile to QIR bitcode", e);
          vscode.window.showErrorMessage(
            "Failed to compile to QIR bitcode: " + e
          );
          return;
        }
      }

      const token = await getTokenForWorkspace(arg.workspace);
      const quantumUris = new QuantumUris(
        arg.workspace.endpointUri,
        arg.workspace.id
      );

      const jobName = await submitJob(
        token,
        quantumUris,
        payload,
        providerId,
        target.id
      );

      // TODO(billti) ensure the workspace tree regularly refreshes while a job is pending completion
      setTimeout(async () => {
        await queryWorkspace(arg.workspace);
        workspaceTreeProvider.updateWorkspace(arg.workspace);
      }, 1000);
    }
  );

  vscode.commands.registerCommand("quantum-workspaces-refresh", () => {
    workspaceTreeProvider.refresh();
  });

  vscode.commands.registerCommand("quantum-workspaces-add", async () => {
    vscode.commands.executeCommand("extension.qsharp.aadSignin");
  });

  vscode.commands.registerCommand("extension.qsharp.aadSignin", async () => {
    const workspace = await queryWorkspaces();
    if (workspace) {
      workspaceTreeProvider.updateWorkspace(workspace);
      workspaceTreeProvider.refresh();
    }
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
