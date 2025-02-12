// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { getTokenForWorkspace } from "../azure/auth.js";
import { QuantumUris } from "../azure/networkRequests.js";
import { supportsAdaptive } from "../azure/providerProperties.js";
import { startRefreshCycle } from "../azure/treeRefresher.js";
import {
  Job,
  Provider,
  Target,
  WorkspaceConnection,
  WorkspaceTreeProvider,
} from "../azure/treeView.js";
import { getJobFiles, submitJob } from "../azure/workspaceActions.js";
import { HistogramData } from "./shared.js";
import { getQirForVisibleQs } from "../qirGeneration.js";
import { CopilotToolError, ToolResult, ToolState } from "./tools.js";

/**
 * These tool definitions correspond to the ones declared
 * service side. Their names and arguments *must* be kept
 * in sync with the service.
 *
 * The return types can be updated independently from the
 * service as they don't have to adhere to a strict schema.
 */
export const azqToolDefinitions: {
  [key: string]: {
    handler: (conversationState: ToolState, args?: any) => Promise<ToolResult>;
    statusMessage: string;
  };
} = {
  GetJobs: { handler: getJobs, statusMessage: "Getting recent jobs" },
  GetJob: { handler: getJob, statusMessage: "Getting job details" },
  ConnectToWorkspace: {
    handler: connectToWorkspace,
    statusMessage: "Connecting to workspace",
  },
  DownloadJobResults: {
    handler: downloadJobResults,
    statusMessage: "Retrieving job results",
  },
  GetWorkspaces: {
    handler: getWorkspaces,
    statusMessage: "Getting available workspaces",
  },
  SubmitToTarget: { handler: submitToTarget, statusMessage: "Submitting job" },
  GetActiveWorkspace: {
    handler: getActiveWorkspace,
    statusMessage: "Getting active workspace",
  },
  SetActiveWorkspace: {
    handler: setActiveWorkspace,
    statusMessage: "Setting the active workspace",
  },
  GetProviders: {
    handler: getProviders,
    statusMessage: "Retrieving list of providers",
  },
  GetTarget: { handler: getTarget, statusMessage: "Getting target details" },
};

/**
 * Filters out unknown tool names that may come back from the service.
 */
export function knownToolNameOrDefault(toolName: string): string {
  return Object.keys(azqToolDefinitions).indexOf(toolName) !== -1
    ? toolName
    : "unknown";
}

/**
 * Gets the first available workspace connection, or throws if there are none.
 */
async function getFirstWorkspace(): Promise<WorkspaceConnection> {
  const tree = WorkspaceTreeProvider.instance;
  const workspaces = tree.getWorkspaceIds();
  const workspace = workspaces[0] && tree.getWorkspace(workspaces[0]);
  if (workspace) {
    return workspace;
  } else {
    throw new CopilotToolError(
      "There are no Azure Quantum workspace connections set up.",
    );
  }
}

/**
 * Gets the current workspace for the conversation,
 * or the first available workspace workspace connection if none has been set.
 * Throws if there are no workspace connections available.
 */
async function getConversationWorkspace(
  toolState: ToolState,
): Promise<WorkspaceConnection> {
  if (toolState.activeWorkspace) {
    return toolState.activeWorkspace;
  } else {
    const initialWorkspaceResult = await getFirstWorkspace();
    toolState.activeWorkspace = initialWorkspaceResult;
    return initialWorkspaceResult;
  }
}

/**
 * Poll for updates on the workspace until the job
 * with the given ID is found and all pending jobs are completed.
 */
function startRefreshingWorkspace(
  workspaceConnection: WorkspaceConnection,
  newJobId?: string,
) {
  startRefreshCycle(
    WorkspaceTreeProvider.instance,
    workspaceConnection,
    newJobId,
  );
}

/**
 * Gets the list of available workspace connections.
 */
async function getWorkspaces(): Promise<{
  result: { workspaceIds: string[] };
}> {
  return {
    result: { workspaceIds: WorkspaceTreeProvider.instance.getWorkspaceIds() },
  };
}

/**
 * Gets the ID of the active workspace for this conversation,
 * or throws if no workspace connections are available.
 */
async function getActiveWorkspace(
  toolState: ToolState,
): Promise<{ result: { workspaceId: string } }> {
  const workspace = await getConversationWorkspace(toolState);
  return { result: { workspaceId: workspace.id } };
}

/**
 * Sets the active workspace for this conversation.
 */
async function setActiveWorkspace(
  toolState: ToolState,
  { workspace_id: workspaceId }: { workspace_id: string },
): Promise<{ result: string }> {
  const tree = WorkspaceTreeProvider.instance;
  const workspace = tree.getWorkspace(workspaceId);
  if (!workspace) {
    throw new CopilotToolError(
      "A workspace with id " + workspaceId + " was not found.",
    );
  } else {
    toolState.activeWorkspace = workspace;
    return { result: "Workspace " + workspaceId + " set as active." };
  }
}

const jobLimit = 10;
const jobLimitDays = 14;

async function getRecentJobs(workspace: WorkspaceConnection): Promise<Job[]> {
  if (workspace) {
    const jobs = workspace.jobs;

    const start = new Date();
    start.setHours(0, 0, 0, 0);
    start.setDate(start.getDate() - jobLimitDays);

    const limitedJobs = (
      jobs.length > jobLimit ? jobs.slice(0, jobLimit) : jobs
    ).filter((j) => new Date(j.creationTime) > start);
    return limitedJobs;
  } else {
    return [];
  }
}

export async function getJobs(conversationState: ToolState): Promise<{
  result: {
    recentJobs: MinimizedJob[];
    lastNJobs: number;
    lastNDays: number;
  };
}> {
  const workspace = await getConversationWorkspace(conversationState);

  const recentJobs = (await getRecentJobs(workspace)).map((job) => {
    return {
      id: job.id,
      name: job.name,
      target: job.target,
      status: job.status,
      count: job.count,
      shots: job.shots,
      creationTime: job.creationTime,
      beginExecutionTime: job.beginExecutionTime,
      endExecutionTime: job.endExecutionTime,
      cancellationTime: job.cancellationTime,
      costEstimate: job.costEstimate,
    };
  });

  return {
    result: { recentJobs, lastNJobs: jobLimit, lastNDays: jobLimitDays },
  };
}

type MinimizedJob = {
  id: string;
  name: string;
  target: string;
  status:
    | "Waiting"
    | "Executing"
    | "Succeeded"
    | "Failed"
    | "Finishing"
    | "Cancelled";
  count: number;
  shots: number;
  creationTime: string;
  beginExecutionTime?: string;
  endExecutionTime?: string;
  cancellationTime?: string;
  costEstimate?: any;
  errorData?: { code: string; message: string };
};

/**
 * Gets job details for the job with the given ID from the active workspace,
 * or throws if the job is not found.
 */
async function getJob(
  toolState: ToolState,
  { job_id }: { job_id: string },
): Promise<{
  result: Job;
}> {
  const workspace = await getConversationWorkspace(toolState);
  const jobs = workspace.jobs;
  const job = jobs.find((job) => job.id === job_id);

  if (!job) {
    throw new CopilotToolError(
      "A job with ID " + job_id + " was not found in the workspace.",
    );
  }

  return { result: job };
}

type DownloadJobResult = {
  result: string;
  widgetData?: HistogramData;
};

/**
 * Download the results of the job with the given ID from the active workspace.
 * Throws if the job can't be found or the results can't be downloaded for any reason.
 */
async function downloadJobResults(
  toolState: ToolState,
  args: { job_id: string },
): Promise<DownloadJobResult> {
  const job = (await getJob(toolState, args)).result;

  if (job.status !== "Succeeded") {
    throw new CopilotToolError("Job has not completed successfully.");
  }

  const workspace = await getConversationWorkspace(toolState);

  if (!job.outputDataUri) {
    log.error("Download called for job with null outputDataUri", job);
    throw new CopilotToolError(
      "Failed to download the output data for the job.",
    );
  }

  const fileUri = vscode.Uri.parse(job.outputDataUri);
  const [, container, blob] = fileUri.path.split("/");

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  const token = await getTokenForWorkspace(workspace);
  if (!token) {
    log.error("Unable to get token for the workspace", workspace);
    throw new CopilotToolError("Failed to connect to the workspace.");
  }

  const outputData = await getJobFiles(container, blob, token, quantumUris);
  if (outputData) {
    const buckets = formatHistogramBuckets(outputData);
    if (!buckets) {
      const doc = await vscode.workspace.openTextDocument({
        content: outputData,
        language: "json",
      });
      vscode.window.showTextDocument(doc);
      return { result: "Results downloaded successfully as file." };
    } else {
      const histogram = {
        buckets,
        shotCount: job.shots,
      };
      return {
        result: "Results were successfully rendered.",
        widgetData: histogram,
      };
    }
  }
  throw new CopilotToolError("Failed to download the results for the job.");
}

/**
 * Convert raw output data from a job to the histogram buckets
 * format we use for display.
 */
function formatHistogramBuckets(
  outputData: string,
): [string, number][] | undefined {
  try {
    // Parse the JSON file
    const parsedArray = JSON.parse(outputData).Histogram as Array<any>;

    if (parsedArray.length % 2 !== 0) {
      // "Data is not in correct format for histogram."
      return undefined;
    }

    // Transform the flat array into an array of pairs [string, number]
    const histo: Array<[string, number]> = [];
    for (let i = 0; i < parsedArray.length; i += 2) {
      histo.push([parsedArray[i], parsedArray[i + 1]]);
    }

    return histo;
  } catch (e: any) {
    log.error("Error rendering results as histogram: ", e);
    return undefined;
  }
}

/**
 * Gets the list of the providers and targets in the current workspace.
 */
async function getProviders(
  toolState: ToolState,
): Promise<{ result: Provider[] }> {
  const workspace = await getConversationWorkspace(toolState);
  return { result: workspace?.providers ?? [] };
}

/**
 * Gets details about a specific target by its name.
 */
async function getTarget(
  toolState: ToolState,
  { target_id }: { target_id: string },
): Promise<{ result: Target | undefined }> {
  const providers = (await getProviders(toolState)).result;
  for (const provider of providers) {
    const target = provider.targets.find((target) => target.id === target_id);
    if (target) {
      return { result: target };
    }
  }
  return { result: undefined };
}

/**
 * Submits the Q# program in the currently visible editor window to Azure Quantum.
 */
async function submitToTarget(
  toolState: ToolState,
  {
    job_name: jobName,
    target_id: target_id,
    number_of_shots: numberOfShots,
  }: { job_name: string; target_id: string; number_of_shots: number },
): Promise<{ result: string }> {
  const target = (await getTarget(toolState, { target_id })).result;
  if (!target) {
    throw new CopilotToolError(
      "A target with the name " +
        target_id +
        " does not exist in the workspace.",
    );
  }

  if (target.currentAvailability !== "Available")
    throw new CopilotToolError(
      "The target " + target_id + " is not available.",
    );

  const workspace = await getConversationWorkspace(toolState);

  const providerId = target.id.split(".")?.[0];

  let qir = "";
  try {
    qir = await getQirForVisibleQs(supportsAdaptive(target.id));
  } catch (e: any) {
    if (e?.name === "QirGenerationError") {
      throw new CopilotToolError(e.message);
    }
  }

  if (!qir) throw new CopilotToolError("Failed to generate QIR.");

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  try {
    const token = await getTokenForWorkspace(workspace);
    if (!token) {
      log.error("Unable to get token for the workspace", workspace);
      throw new CopilotToolError("Failed to connect to the workspace.");
    }

    const jobId = await submitJob(
      token,
      quantumUris,
      qir,
      providerId,
      target.id,
      jobName,
      numberOfShots,
    );
    startRefreshingWorkspace(workspace, jobId);
    return { result: "Job submitted successfully with ID: " + jobId };
  } catch (e: any) {
    log.error("Failed to submit job. ", e);
    const error = e instanceof Error ? e.message : "";

    throw new CopilotToolError("Failed to submit the job. " + error);
  }
}

/**
 * Starts the user flow to connect to an Azure Quantum Workspace.
 */
async function connectToWorkspace(
  conversationState: ToolState,
): Promise<{ result: string }> {
  const initialWsList = await getWorkspaces();
  try {
    await vscode.commands.executeCommand("qsharp-vscode.workspacesAdd");
  } catch {
    throw new CopilotToolError(
      `An error occurred while trying to connect to an Azure Quantum Workspace.`,
    );
  }
  const newWsList = await getWorkspaces();
  // Kind of a silly way to do this, but the new workspace is the one
  // that exists in `newWsList` but not in `initialWsList`
  const newWorkspaceId = newWsList.result.workspaceIds.filter(
    (id: string) => !initialWsList.result.workspaceIds.includes(id),
  )[0];

  if (newWorkspaceId) {
    await setActiveWorkspace(conversationState, {
      workspace_id: newWorkspaceId,
    });

    return {
      result:
        "Connected to Azure Quantum Workspace with ID `" + newWorkspaceId + "`",
    };
  }

  return {
    result:
      "A new workspace was not added. Available workspaces: " +
      JSON.stringify(newWsList),
  };
}
