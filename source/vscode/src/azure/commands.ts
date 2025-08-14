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
  queryWorkspaces,
  submitJob,
} from "./workspaceActions";
import { QuantumUris } from "./networkRequests";
import { getQirForActiveWindow } from "../qirGeneration";
import {
  getPreferredTargetProfile,
  targetSupportQir,
} from "./providerProperties";
import { startRefreshCycle } from "./treeRefresher";
import { getTokenForWorkspace } from "./auth";
import { qsharpExtensionId } from "../common";
import { sendMessageToPanel } from "../webviewPanel";

const workspacesSecret = `${qsharpExtensionId}.workspaces`;

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
        `${qsharpExtensionId}.treeItemSupportsQir`,
        supportsQir,
      );
      await vscode.commands.executeCommand(
        "setContext",
        `${qsharpExtensionId}.treeItemSupportsDownload`,
        supportsDownload,
      );
      await vscode.commands.executeCommand(
        "setContext",
        `${qsharpExtensionId}.treeItemIsWorkspace`,
        isWorkspace,
      );
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.targetSubmit`,
      async (arg: WorkspaceTreeItem) => {
        // Could be run via the treeItem icon or the menu command.
        const treeItem = arg || currentTreeItem;
        if (treeItem?.type !== "target") return;

        const target = treeItem.itemData as Target;

        let qir = "";
        try {
          qir = await getQirForActiveWindow(
            getPreferredTargetProfile(target.id),
          );
        } catch (e: any) {
          if (e?.name === "QirGenerationError") {
            vscode.window.showErrorMessage(e.message, { modal: true });
            return;
          }
        }
        if (!qir) return;

        const quantumUris = new QuantumUris(
          treeItem.workspace.endpointUri,
          treeItem.workspace.id,
        );

        try {
          const token = await getTokenForWorkspace(treeItem.workspace);
          if (!token) return;

          const jobId = await submitJob(
            token,
            quantumUris,
            qir,
            target.providerId,
            target.id,
          );
          if (jobId) {
            // The job submitted fine. Refresh the workspace until it shows up
            // and all jobs are finished. Don't await on this, just let it run
            startRefreshCycle(workspaceTreeProvider, treeItem.workspace, jobId);
          }
        } catch (e: any) {
          log.error("Failed to submit job. ", e);

          vscode.window.showErrorMessage("Failed to submit the job to Azure.", {
            modal: true,
            detail: e instanceof Error ? e.message : undefined,
          });
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.workspacesRefresh`,
      () => {
        // The user manually triggered a refresh. Start a cycle for each workspace
        const workspaceIds = workspaceTreeProvider.getWorkspaceIds();

        workspaceIds.forEach((id) => {
          const workspace = workspaceTreeProvider.getWorkspace(id);
          if (workspace) {
            startRefreshCycle(workspaceTreeProvider, workspace);
          }
        });
      },
    ),
  );

  async function saveWorkspaceList() {
    // Save the list of workspaces
    const savedWorkspaces: WorkspaceConnection[] = [];
    const workspaces = workspaceTreeProvider
      .getWorkspaceIds()
      .map((id) => workspaceTreeProvider.getWorkspace(id)!);

    for (const elem of workspaces) {
      // Save only the general workspace information, not the providers and jobs
      savedWorkspaces.push({
        id: elem.id,
        name: elem.name,
        endpointUri: elem.endpointUri,
        tenantId: elem.tenantId,
        apiKey: elem.apiKey,
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
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.workspacesAdd`,
      async () => {
        const workspace = await queryWorkspaces();
        if (workspace) {
          workspaceTreeProvider.updateWorkspace(workspace);
          await saveWorkspaceList();
          // Just kick off the refresh loop, no need to await
          startRefreshCycle(workspaceTreeProvider, workspace);
        }
      },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.workspacesRemove`,
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

  async function downloadResults(arg?: WorkspaceTreeItem, showText?: boolean) {
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

    const quantumUris = new QuantumUris(
      treeItem.workspace.endpointUri,
      treeItem.workspace.id,
    );

    try {
      const token = await getTokenForWorkspace(treeItem.workspace);
      if (!token) return;

      const file = await getJobFiles(container, blob, token, quantumUris);
      const buckets = !showText && getHistogramBucketsFromData(file, job.shots);
      if (buckets) {
        sendMessageToPanel({ panelType: "histogram", id: job.name }, true, {
          ...buckets,
          suppressSettings: true, // Don't want to show noise settings on downloaded results
        });
      } else {
        if (!showText)
          vscode.window.showInformationMessage(
            "Unable to display results as a histogram. Opening as text.",
          );
        const doc = await vscode.workspace.openTextDocument({
          content: file,
          language: "json",
        });
        vscode.window.showTextDocument(doc);
      }
    } catch (e: any) {
      log.error("Failed to download result file. ", e);
      vscode.window.showErrorMessage("Failed to download the results file.", {
        modal: true,
        detail: e instanceof Error ? e.message : undefined,
      });
    }
  }

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.downloadResults`,
      async (arg: WorkspaceTreeItem) => await downloadResults(arg, false),
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.downloadRawResults`,
      async (arg: WorkspaceTreeItem) => await downloadResults(arg, true),
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.workspacePythonCode`,
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
      `${qsharpExtensionId}.workspaceOpenPortal`,
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

type Buckets = {
  buckets: [string, number][];
  shotCount: number;
};
function getHistogramBucketsFromData(
  file: string,
  shotCount?: number,
): Buckets | undefined {
  try {
    const parsed = JSON.parse(file);
    if (!parsed || typeof parsed !== "object") {
      throw "Histogram data is not in the expected format";
    }
    if (parsed.DataFormat === "microsoft.quantum-results.v2") {
      // New 'v2' format will be in the format
      // {
      //   "DataFormat": "microsoft.quantum-results.v2",
      //   "Results": [
      //     {
      //       "Histogram": [
      //         { "Outcome": [0, 1, 1, 1], "Display": "[0, 1, 1, 1]", "Count": 8 },
      //         { "Outcome": [1, 1, 0, 0], "Display": "[1, 1, 0, 0]", "Count": 10 },
      // etc..
      // Only Results[0] is used (batching may have more entries, but we don't support that)
      type v2Bucket = { Display: string; Count: number };
      const histogramData: v2Bucket[] = parsed.Results?.[0]?.Histogram;
      if (!Array.isArray(histogramData)) throw "Histogram data not found";
      const buckets: Array<[string, number]> = [];
      let shotTotal = 0;
      histogramData.forEach((entry) => {
        shotTotal += entry.Count;
        buckets.push([entry.Display, entry.Count]);
      });

      return { buckets, shotCount: shotTotal };
    } else if (Array.isArray(parsed.Histogram)) {
      // v1 format should be an object with a "Histogram" property, which is an array of ["label", <float>, ...] entries.
      // e.g., something as simple as: {"Histogram":["(0, 0)",0.54,"(1, 1)",0.46]}
      // Note this doesn't include the shotCount, so we need it from the job
      if (!shotCount || !Number.isInteger(shotCount)) {
        throw "job.shots data was not a positive integer";
      }

      // Turn the flat histogram list into buckets for the histogram.
      const histogram: Array<string | number> = parsed.Histogram;
      const buckets: Array<[string, number]> = [];
      for (let i = 0; i < parsed.Histogram.length; i += 2) {
        const label = histogram[i].toString();
        const value = histogram[i + 1]; // This is a percentage, not a count
        if (typeof value !== "number") throw "Invalid histogram value";
        buckets.push([label, Math.round(value * shotCount)]);
      }

      return { buckets, shotCount };
    } else {
      throw "Unrecognized histogram file format";
    }
  } catch (e: any) {
    log.debug("Failed to parse job results as histogram data.", file, e);
  }
  return undefined;
}
