// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp-lang";

import {
  Job,
  Target,
  WorkspaceConnection,
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

const corsDocsUri = "https://github.com/microsoft/qsharp/wiki/Enabling-CORS";
const workspacesSecret = "qsharp-vscode.workspaces";

export async function initAzureWorkspaces(context: vscode.ExtensionContext) {
  const workspaceTreeProvider = new WorkspaceTreeProvider();
  vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });

  // Add any previously saved workspaces
  const savedWorkspaces = await context.secrets.get(workspacesSecret);
  if (savedWorkspaces) {
    log.debug("Loading workspaces: ", savedWorkspaces);
    const workspaces: WorkspaceConnection[] = JSON.parse(savedWorkspaces);
    for (const workspace of workspaces) {
      await queryWorkspace(workspace); // Fetch the providers and jobs
      workspaceTreeProvider.updateWorkspace(workspace);
    }
  } else {
    log.debug("No saved workspaces found.");
  }

  vscode.commands.registerCommand(
    "qsharp-vscode.targetSubmit",
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

      setTimeout(() => {
        workspaceTreeProvider.refresh();
      }, 2000);
    }
  );

  vscode.commands.registerCommand("qsharp-vscode.workspacesRefresh", () => {
    workspaceTreeProvider.refresh();
  });

  async function saveWorkspaceList() {
    // Save the list of workspaces
    const savedWorkspaces: WorkspaceConnection[] = [];
    for (const elem of workspaceTreeProvider.treeState.values()) {
      // Save only the general workspace information, not the providers and jobs
      savedWorkspaces.push({
        id: elem.id,
        name: elem.name,
        endpointUri: elem.endpointUri,
        tenantId: elem.tenantId,
        providers: [],
        jobs: [],
      });
    }
    log.debug("Saving workspaces: ", savedWorkspaces);
    await context.secrets.store(
      workspacesSecret,
      JSON.stringify(savedWorkspaces)
    );
  }

  vscode.commands.registerCommand("qsharp-vscode.workspacesAdd", async () => {
    const workspace = await queryWorkspaces();
    if (workspace) {
      queryWorkspace(workspace); // To fetch the providers and jobs
      workspaceTreeProvider.updateWorkspace(workspace);
      await saveWorkspaceList();
    }
  });

  vscode.commands.registerCommand(
    "qsharp-vscode.workspacesRemove",
    async (item: WorkspaceTreeItem) => {
      if (!item || item.type !== "workspace") return;
      const workspace = item.itemData as WorkspaceConnection;
      workspaceTreeProvider.treeState.delete(workspace.id);
      await saveWorkspaceList();
      await workspaceTreeProvider.refresh();
    }
  );

  vscode.commands.registerCommand(
    "qsharp-vscode.downloadResults",
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
