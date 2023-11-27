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
  getAzurePortalWorkspaceLink,
  getJobFiles,
  getPythonCodeForWorkspace,
  getTokenForWorkspace,
  queryWorkspaces,
  submitJob,
} from "./workspaceActions";
import {
  QuantumUris,
  checkCorsConfig,
  compileToBitcode,
  useProxy,
} from "./networkRequests";
import { getQirForActiveWindow } from "../qirGeneration";
import { targetSupportQir } from "./providerProperties";
import { startRefreshCycle } from "./treeRefresher";

const corsDocsUri = "https://aka.ms/qdk.cors";
const workspacesSecret = "qsharp-vscode.workspaces";

export async function initAzureWorkspaces(context: vscode.ExtensionContext) {
  const workspaceTreeProvider = new WorkspaceTreeProvider();
  WorkspaceTreeProvider.instance = workspaceTreeProvider;

  const treeView = vscode.window.createTreeView("quantum-workspaces", {
    treeDataProvider: workspaceTreeProvider,
  });
  context.subscriptions.push(treeView);

  let currentTreeItem: WorkspaceTreeItem | undefined = undefined;

  // Add any previously saved workspaces
  const savedWorkspaces = await context.secrets.get(workspacesSecret);
  if (savedWorkspaces) {
    log.debug("Loading workspaces: ", savedWorkspaces);
    const workspaces: WorkspaceConnection[] = JSON.parse(savedWorkspaces);
    for (const workspace of workspaces) {
      workspaceTreeProvider.updateWorkspace(workspace);
      // Start refreshing each workspace until pending jobs are complete
      startRefreshCycle(workspaceTreeProvider, workspace);
    }
  } else {
    log.debug("No saved workspaces found.");
  }

  context.subscriptions.push(
    treeView.onDidChangeSelection(async (e) => {
      // Capture the selected item and set context if the supports job submission or results download.
      let supportsQir = false;
      let supportsDownload = false;
      let isWorkspace = false;

      if (e.selection.length === 1) {
        currentTreeItem = e.selection[0] as WorkspaceTreeItem;
        if (
          currentTreeItem.type === "target" &&
          targetSupportQir(currentTreeItem.label?.toString() || "")
        ) {
          supportsQir = true;
        }
        if (currentTreeItem.type === "job") {
          const job = currentTreeItem.itemData as Job;
          if (job.status === "Succeeded" && job.outputDataUri) {
            supportsDownload = true;
          }
        }
        if (currentTreeItem.type === "workspace") {
          isWorkspace = true;
        }
      } else {
        currentTreeItem = undefined;
      }

      await vscode.commands.executeCommand(
        "setContext",
        "qsharp-vscode.treeItemSupportsQir",
        supportsQir,
      );
      await vscode.commands.executeCommand(
        "setContext",
        "qsharp-vscode.treeItemSupportsDownload",
        supportsDownload,
      );
      await vscode.commands.executeCommand(
        "setContext",
        "qsharp-vscode.treeItemIsWorkspace",
        isWorkspace,
      );
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.targetSubmit",
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "target") return;

        const target = treeItem.itemData as Target;

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

        const token = await getTokenForWorkspace(treeItem.workspace);
        if (!token) return;

        const quantumUris = new QuantumUris(
          treeItem.workspace.endpointUri,
          treeItem.workspace.id,
        );

        try {
          const jobId = await submitJob(
            token,
            quantumUris,
            payload,
            providerId,
            target.id,
          );
          if (jobId) {
            // The job submitted fine. Refresh the workspace until it shows up
            // and all jobs are finished. Don't await on this, just let it run
            startRefreshCycle(workspaceTreeProvider, treeItem.workspace, jobId);
          }
        } catch (e: any) {
          log.error("Failed to submit job. ", e);
          vscode.window.showErrorMessage(
            "Failed to submit the job to Azure. " +
              "Ensure CORS is configured correctly on the workspace storage account. " +
              `See ${corsDocsUri} for more information.`,
          );
          return;
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.workspacesRefresh", () => {
      // The user manually triggered a refresh. Start a cycle for each workspace
      const workspaceIds = workspaceTreeProvider.getWorkspaceIds();

      workspaceIds.forEach((id) => {
        const workspace = workspaceTreeProvider.getWorkspace(id);
        if (workspace) {
          startRefreshCycle(workspaceTreeProvider, workspace);
        }
      });
    }),
  );

  async function saveWorkspaceList() {
    // Save the list of workspaces
    const savedWorkspaces: WorkspaceConnection[] = [];
    const workspaces = workspaceTreeProvider
      .getWorkspaceIds()
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      .map((id) => workspaceTreeProvider.getWorkspace(id)!);

    for (const elem of workspaces) {
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
      JSON.stringify(savedWorkspaces),
    );
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.workspacesAdd", async () => {
      const workspace = await queryWorkspaces();
      if (workspace) {
        workspaceTreeProvider.updateWorkspace(workspace);
        await saveWorkspaceList();
        // Just kick off the refresh loop, no need to await
        startRefreshCycle(workspaceTreeProvider, workspace);

        // Check if the storage account has CORS configured correctly.
        // NOTE: This should be removed once talking directly to Azure storage is no longer required.
        const quantumUris = new QuantumUris(
          workspace.endpointUri,
          workspace.id,
        );

        const token = await getTokenForWorkspace(workspace);
        if (!token) return;
        try {
          if (!useProxy) await checkCorsConfig(token, quantumUris);
        } catch (e: any) {
          log.debug("CORS check failed. ", e);

          const selection = await vscode.window.showWarningMessage(
            "The Quantum Workspace added needs to have CORS configured to be able to submit jobs or fetch results. " +
              `Would you like to visit the documentation page at ${corsDocsUri} for details on how to configure this?`,
            { modal: true },
            { title: "Open CORS documentation", action: "open" },
            { title: "Cancel", action: "cancel", isCloseAffordance: true },
          );
          if (selection?.action === "open") {
            vscode.env.openExternal(vscode.Uri.parse(corsDocsUri));
          }
        }
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.workspacesRemove",
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "workspace") return;
        const workspace = treeItem.itemData as WorkspaceConnection;

        workspaceTreeProvider.removeWorkspace(workspace.id);
        await saveWorkspaceList();
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.downloadResults",
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "job") return;

        const job = treeItem.itemData as Job;

        if (!job.outputDataUri) {
          log.error("Download called for job with null outputDataUri", job);
          return;
        }

        const fileUri = vscode.Uri.parse(job.outputDataUri);
        const [, container, blob] = fileUri.path.split("/");

        const token = await getTokenForWorkspace(treeItem.workspace);
        if (!token) return;

        const quantumUris = new QuantumUris(
          treeItem.workspace.endpointUri,
          treeItem.workspace.id,
        );

        try {
          const file = await getJobFiles(container, blob, token, quantumUris);
          if (file) {
            const doc = await vscode.workspace.openTextDocument({
              content: file,
              language: "json",
            });
            vscode.window.showTextDocument(doc);
          }
        } catch (e: any) {
          log.error("Failed to download result file. ", e);
          vscode.window.showErrorMessage(
            "Failed to download the results file. " +
              "Ensure CORS is configured correctly on the workspace storage account. " +
              `See ${corsDocsUri} for more information.`,
          );
          return;
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.workspacePythonCode",
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "workspace") return;
        const workspace = treeItem.itemData as WorkspaceConnection;
        const str = getPythonCodeForWorkspace(
          workspace.id,
          workspace.endpointUri,
          workspace.name,
        );
        if (str) {
          vscode.env.clipboard.writeText(str);
          vscode.window.showInformationMessage(
            "Python code has been copied to the clipboard",
          );
        } else {
          vscode.window.showErrorMessage(
            "Failed to generate Python code for workspace",
          );
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      "qsharp-vscode.workspaceOpenPortal",
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "workspace") return;
        const workspace = treeItem.itemData as WorkspaceConnection;

        const link = getAzurePortalWorkspaceLink(workspace);
        vscode.env.openExternal(vscode.Uri.parse(link));
      },
    ),
  );
}
