// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { log } from "qsharp-lang";
import {
  Job,
  Provider,
  Target,
  WorkspaceConnection,
  WorkspaceTreeProvider,
} from "../azure/treeView.js";
import { QuantumUris } from "../azure/networkRequests.js";
import { getTokenForWorkspace } from "../azure/auth.js";
import {
  getJobFiles,
  submitJobWithNameAndShots,
} from "../azure/workspaceActions.js";
import { supportsAdaptive } from "../azure/providerProperties.js";
import { getQirForVisibleQs } from "../qirGeneration.js";
import { startRefreshCycle } from "../azure/treeRefresher.js";
import { ConversationState } from "./copilot.js";
import { handleGetJobs } from "./toolGetJobs.js";
import { handleConnectToWorkspace } from "./toolAddWorkspace.js";

// Define the tools and system prompt that the model can use

/**
 * The messages from these exceptions will be added to the conversation
 * history, so keep the messages meaningful to the copilot and/or user.
 */
class CopilotToolError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CopilotToolError";
  }
}

// Gets the first workspace in the tree, if there is one
export async function getInitialWorkspace(): Promise<WorkspaceConnection> {
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

// Gets the workspace for the conversation, or the first workspace if none is active
export async function getConversationWorkspace(
  toolState: ConversationState,
): Promise<WorkspaceConnection> {
  if (toolState.activeWorkspace) {
    return toolState.activeWorkspace;
  } else {
    const initialWorkspaceResult = await getInitialWorkspace();
    toolState.activeWorkspace = initialWorkspaceResult;
    return initialWorkspaceResult;
  }
}

export function startRefreshingWorkspace(
  workspaceConnection: WorkspaceConnection,
  newJobId?: string,
) {
  startRefreshCycle(
    WorkspaceTreeProvider.instance,
    workspaceConnection,
    newJobId,
  );
}

export const GetWorkspaces = async (): Promise<string[]> => {
  const tree = WorkspaceTreeProvider.instance;
  return tree.getWorkspaceIds();
};

export const GetActiveWorkspace = async (
  toolState: ConversationState,
): Promise<string> => {
  const workspace = await getConversationWorkspace(toolState);
  if (!workspace) {
    return "No active workspace found.";
  }
  return workspace.id;
};

export const SetActiveWorkspace = async (
  workspaceId: string,
  toolState: ConversationState,
): Promise<string> => {
  const tree = WorkspaceTreeProvider.instance;
  const workspace = tree.getWorkspace(workspaceId);
  if (!workspace) {
    return "Workspace not found.";
  } else {
    toolState.activeWorkspace = workspace;
    return "Workspace " + workspaceId + " set as active.";
  }
};

async function getJob(
  jobId: string,
  toolState: ConversationState,
): Promise<Job | undefined> {
  const workspace = await getConversationWorkspace(toolState);
  if (workspace) {
    const jobs = workspace.jobs;

    return jobs.find((job) => job.id === jobId);
  } else {
    return undefined;
  }
}

function tryRenderResults(file: string): [string, number][] | undefined {
  try {
    // Parse the JSON file
    const parsedArray = JSON.parse(file).Histogram as Array<any>;

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

export async function downloadJobResults(
  jobId: string,
  toolState: ConversationState,
): Promise<string> {
  const job = await getJob(jobId, toolState);

  if (!job) {
    log.error("Failed to find the job.");
    return "Failed to find the job.";
  }

  if (job.status !== "Succeeded") {
    return "Job has not completed successfully.";
  }

  const workspace = await getConversationWorkspace(toolState);

  if (!workspace) {
    log.error("Failed to find the workspace.");
    return "Failed to find the workspace.";
  }

  if (!job.outputDataUri) {
    log.error("Download called for job with null outputDataUri", job);
    return "Failed to download the results file.";
  }

  const fileUri = vscode.Uri.parse(job.outputDataUri);
  const [, container, blob] = fileUri.path.split("/");

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  try {
    const token = await getTokenForWorkspace(workspace);
    if (!token) {
      log.error("Unable to get token for the workspace", workspace);
      return "Failed to download the results file.";
    }

    const file = await getJobFiles(container, blob, token, quantumUris);
    if (file) {
      const histo = tryRenderResults(file);
      if (histo === undefined) {
        const doc = await vscode.workspace.openTextDocument({
          content: file,
          language: "json",
        });
        vscode.window.showTextDocument(doc);
        return "Results downloaded successfully as file.";
      } else {
        toolState.sendMessage({
          payload: {
            buckets: histo,
            shotCount: job.shots,
          },
          kind: "copilotResponseHistogram",
        });
        return "Results rendered successfully.";
      }
    }
    return "Failed to get the results file for the job.";
  } catch (e: any) {
    log.error("Failed to download result file. ", e);
    vscode.window.showErrorMessage("Failed to download the results file.", {
      modal: true,
      detail: e instanceof Error ? e.message : undefined,
    });
    return "Failed to download the results file.";
  }
}

export async function GetProviders(
  toolState: ConversationState,
): Promise<Provider[]> {
  const workspace = await getConversationWorkspace(toolState);
  return workspace?.providers ?? [];
}

export const GetTarget = async (
  targetId: string,
  toolState: ConversationState,
): Promise<Target | undefined> => {
  const providers = await GetProviders(toolState);
  for (const provider of providers) {
    const target = provider.targets.find((target) => target.id === targetId);
    if (target) {
      return target;
    }
  }
};

export async function submitToTarget(
  jobName: string,
  targetId: string,
  numberOfShots: number,
  toolState: ConversationState,
): Promise<string> {
  const target = await GetTarget(targetId, toolState);
  if (!target || target.currentAvailability !== "Available")
    return "Target not available.";

  const workspace = await getConversationWorkspace(toolState);

  if (!workspace) {
    log.error("Failed to find the workspace.");
    return "Failed to find the workspace.";
  }

  const providerId = target.id.split(".")?.[0];

  const supports_adaptive = supportsAdaptive(target.id);

  let qir = "";
  try {
    qir = await getQirForVisibleQs(supports_adaptive);
  } catch (e: any) {
    if (e?.name === "QirGenerationError") {
      vscode.window.showErrorMessage(e.message);
      return "Error: " + e.message;
    }
  }
  if (!qir) return "Failed to generate QIR.";

  const quantumUris = new QuantumUris(workspace.endpointUri, workspace.id);

  try {
    const token = await getTokenForWorkspace(workspace);
    if (!token) {
      log.error("Unable to get token for the workspace", workspace);
      return "Failed to download the results file.";
    }

    const jobId = await submitJobWithNameAndShots(
      token,
      quantumUris,
      qir,
      providerId,
      target.id,
      jobName,
      numberOfShots,
    );
    startRefreshingWorkspace(workspace, jobId);
    return "Job submitted successfully with ID: " + jobId;
  } catch (e: any) {
    log.error("Failed to submit job. ", e);
    const error = e instanceof Error ? e.message : undefined;

    vscode.window.showErrorMessage("Failed to submit the job to Azure.", {
      modal: true,
      detail: error,
    });
    return "Failed to submit the job. " + error;
  }
}

const toolHandlers: {
  [key: string]: (conversationState: ConversationState) => object;
} = {
  GetJobs: handleGetJobs,
  ConnectToWorkspace: handleConnectToWorkspace,
};

export async function executeTool(
  tool_name: string,
  args: any,
  toolState: ConversationState,
): Promise<any> {
  const content: any = {};

  log.info("Tool call name: ", tool_name);
  log.info("Tool call args: ", args);

  const handler = toolHandlers[tool_name];
  if (handler) {
    try {
      // Ignore the IDE suggestions, the `await` here is crucial
      // as the exception will not be handled if the promise is not awaited.
      return await handler(toolState);
    } catch (e) {
      if (e instanceof CopilotToolError) {
        return { error: e.message };
      }
    }
  }

  if (tool_name === "GetJob") {
    const job_id = args.job_id;
    const job = await getJob(job_id, toolState);
    content.job = job;
  } else if (tool_name === "GetWorkspaces") {
    const workspace_ids = await GetWorkspaces();
    content.workspace_ids = workspace_ids;
  } else if (tool_name === "GetActiveWorkspace") {
    const active_workspace = await GetActiveWorkspace(toolState);
    content.active_workspace = active_workspace;
  } else if (tool_name === "SetActiveWorkspace") {
    const workspace_id = args.workspace_id;
    const result = await SetActiveWorkspace(workspace_id, toolState);
    content.result = result;
  } else if (tool_name === "DownloadJobResults") {
    const job_id = args.job_id;
    const download_result = await downloadJobResults(job_id, toolState);
    content.download_result = download_result;
  } else if (tool_name === "GetProviders") {
    const providers = await GetProviders(toolState);
    content.providers = providers;
  } else if (tool_name === "GetTarget") {
    const target_id = args.target_id;
    const target = await GetTarget(target_id, toolState);
    content.target = target;
  } else if (tool_name === "SubmitToTarget") {
    const job_name = args.job_name;
    const target_id = args.target_id;
    const number_of_shots = args.number_of_shots;
    const submit_result = await submitToTarget(
      job_name,
      target_id,
      number_of_shots,
      toolState,
    );
    content.submit_result = submit_result;
  }

  return content;
}
