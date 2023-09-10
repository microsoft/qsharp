// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp-lang";

import {
  Job,
  Target,
  WorkspaceTreeItem,
  WorkspaceTreeProvider,
} from "./treeView";
import {
  getJobFiles,
  getTokenForWorkspace,
  queryWorkspace,
  queryWorkspaces,
  submitJob,
} from "./workspaceActions";
import { QuantumUris, compileToBitcode } from "./networkRequests";
import { getQirForActiveWindow } from "../qirGeneration";

// TODO: Proper docs on the qsharp wiki
const corsDocsUri =
  "https://gist.github.com/billti/09637269db4bae86c0e3a552dd20eb9b";

export function initAzureWorkspaces() {
  const workspaceTreeProvider = new WorkspaceTreeProvider();
  vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  vscode.commands.registerCommand(
    "quantum-target-submit",
    async (arg: WorkspaceTreeItem) => {
      const target = arg.itemData as Target;
      const providerId = target.id.split(".")?.[0];

      let qir = "";
      try {
        qir = await getQirForActiveWindow();
      } catch (e: any) {
        if (e?.name === "QirGenerationError") {
          vscode.window.showErrorMessage(e.message);
          return;
        }
      }
      if (!qir) return;

      // Note: Below compilation to be removed when all regions support .ll text directly
      const compilerService: string | undefined = vscode.workspace
        .getConfiguration("Q#")
        .get("compilerService"); // e.g. in settings.json: "Q#.compilerService": "https://qsx-proxy.azurewebsites.net/api/compile"

      const payload = !compilerService
        ? qir
        : await compileToBitcode(compilerService, qir, providerId);
      // End of compilation to be removed

      const token = await getTokenForWorkspace(arg.workspace);
      if (!token) return;

      const quantumUris = new QuantumUris(
        arg.workspace.endpointUri,
        arg.workspace.id
      );

      try {
        await submitJob(token, quantumUris, payload, providerId, target.id);
      } catch (e: any) {
        log.error("Failed to submit job. ", e);
        vscode.window.showErrorMessage(
          "Failed to submit the job to Azure. " +
            "Ensure CORS is configured correctly on the workspace storage account. " +
            `See ${corsDocsUri} for more information.`
        );
        return;
      }

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

      if (!job.outputDataUri) {
        log.error("Download called for job with null outputDataUri", job);
        return;
      }

      const fileUri = vscode.Uri.parse(job.outputDataUri);
      const [, container, blob] = fileUri.path.split("/");

      const token = await getTokenForWorkspace(arg.workspace);
      if (!token) return;

      const quantumUris = new QuantumUris(
        arg.workspace.endpointUri,
        arg.workspace.id
      );

      try {
        const file = await getJobFiles(container, blob, token, quantumUris);
        if (file) {
          const doc = await vscode.workspace.openTextDocument({
            content: file,
            language: "plaintext",
          });
          vscode.window.showTextDocument(doc);
        }
      } catch (e: any) {
        log.error("Failed to download result file. ", e);
        vscode.window.showErrorMessage(
          "Failed to download the results file. " +
            "Ensure CORS is configured correctly on the workspace storage account. " +
            `See ${corsDocsUri} for more information.`
        );
        return;
      }
    }
  );
}
