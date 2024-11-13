import * as vscode from "vscode";
import { log } from "qsharp-lang";
import { ChatCompletionTool } from "openai/resources/chat/completions";
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
import { CopilotConversation } from "./copilot.js";
import { get } from "http";

export type CopilotStreamCallback = (
  msgPayload: object,
  msgCommand:
    | "copilotResponse"
    | "copilotResponseDone"
    | "copilotResponseHistogram",
) => void;

// Define the tools and system prompt that the model can use

export const CopilotToolsDescriptions: ChatCompletionTool[] = [
  {
    type: "function",
    function: {
      name: "GetJob",
      description:
        "Get the job information for a customer's job. Call this whenever you need to know information about a specific job, for example when a customer asks 'What is the status of my job?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          job_id: {
            type: "string",
            description: "The customer's job ID.",
          },
        },
        required: ["job_id"],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "GetJobs",
      description:
        "Get a list of recent jobs that have been run by the customer, along with their statuses. Call this when you need to know what jobs have been run recently or need a history of jobs run, for example when a customer asks 'What are my recent jobs?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {},
        required: [],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "GetWorkspaces",
      description:
        "Get a list of workspaces the customer has access to, in the form of workspace ids. Call this when you need to know what workspaces the customer has access to, for example when a customer asks 'What are my workspaces?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {},
        required: [],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "GetActiveWorkspace",
      description:
        "Get the id of the active workspace for this conversation. " +
        "Call this when you need to know what workspace is the active workspace being used in the context of the current conversation, " +
        "for example when a customer asks 'What is the workspace that's being used?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {},
        required: [],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "SetActiveWorkspace",
      description:
        "Set the active workspace for this conversation by id. " +
        "Call this when you need to set what workspace is the active workspace being used in the context of the current conversation, " +
        "for example when a customer says 'Please use this workspace for my requests.'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          workspace_id: {
            type: "string",
            description: "The id of the workspace to set as active.",
          },
        },
        required: ["workspace_id"],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "DownloadJobResults",
      description:
        "Download and display the results from a customer's job. " +
        "Call this when you need to download or display as a histogram the results for a job, " +
        "for example when a customer asks 'What are the results of my last job?' " +
        "or 'Can you show me the histogram for this job?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          job_id: {
            type: "string",
            description: "The customer's job ID.",
          },
        },
        required: ["job_id"],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "GetProviders",
      description:
        "Get a list of hardware providers available to the customer, along with their provided targets. Call this when you need to know what providers or targets are available, for example when a customer asks 'What are the available providers?' or 'What targets do I have available?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {},
        required: [],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "GetTarget",
      description:
        "Get the target information for a specified target. Call this whenever you need to know information about a specific target, for example when a customer asks 'What is the status of this target?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          target_id: {
            type: "string",
            description: "The ID of the target to get.",
          },
        },
        required: ["target_id"],
        additionalProperties: false,
      },
    },
  },
  {
    type: "function",
    function: {
      name: "SubmitToTarget",
      description:
        "Submit the current Q# program to Azure Quantum with the provided information. Call this when you need to submit or run a Q# program with Azure Quantum, for example when a customer asks 'Can you submit this program to Azure?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          job_name: {
            type: "string",
            description: "The string to name the created job.",
          },
          target_id: {
            type: "string",
            description: "The ID or name of the target to submit the job to.",
          },
          number_of_shots: {
            type: "number",
            description: "The number of shots to use for the job.",
          },
        },
        required: ["job_name", "target_id", "number_of_shots"],
        additionalProperties: false,
      },
    },
  },
];

const job_limit = 10;

export const GetJobs = async (
  conversation: CopilotConversation,
): Promise<Job[]> => {
  const workspace = await getConversationWorkspace(conversation);
  if (workspace) {
    const jobs = workspace.jobs;

    const limited_jobs =
      jobs.length > job_limit ? jobs.slice(0, job_limit) : jobs;

    return limited_jobs;
  } else {
    return [];
  }
};

// Gets the first workspace in the tree, if there is one
const getInitialWorkspace = async (): Promise<
  WorkspaceConnection | undefined
> => {
  const tree = WorkspaceTreeProvider.instance;
  const workspaces = tree.getWorkspaceIds();
  const workspace = workspaces[0] || undefined;
  if (workspace) {
    return tree.getWorkspace(workspace);
  } else {
    return undefined;
  }
};

// Gets the workspace for the conversation, or the first workspace if none is active
const getConversationWorkspace = async (
  conversation: CopilotConversation,
): Promise<WorkspaceConnection | undefined> => {
  if (conversation.active_workspace) {
    return conversation.active_workspace;
  } else {
    const init_workspace = await getInitialWorkspace();
    conversation.active_workspace = init_workspace;
    return init_workspace;
  }
};

export const GetWorkspaces = async (): Promise<string[]> => {
  const tree = WorkspaceTreeProvider.instance;
  return tree.getWorkspaceIds();
};

export const GetActiveWorkspace = async (
  conversation: CopilotConversation,
): Promise<string> => {
  const workspace = await getConversationWorkspace(conversation);
  if (!workspace) {
    return "No active workspace found.";
  }
  return workspace.id;
};

export const SetActiveWorkspace = async (
  workspaceId: string,
  conversation: CopilotConversation,
): Promise<string> => {
  const tree = WorkspaceTreeProvider.instance;
  const workspace = tree.getWorkspace(workspaceId);
  if (!workspace) {
    return "Workspace not found.";
  } else {
    conversation.active_workspace = workspace;
    return "Workspace " + workspaceId + " set as active.";
  }
};

const GetJob = async (
  jobId: string,
  conversation: CopilotConversation,
): Promise<Job | undefined> => {
  const jobs = await GetJobs(conversation);
  return jobs.find((job) => job.id === jobId);
};

const tryRenderResults = (
  file: string,
  shots: number,
  streamCallback: CopilotStreamCallback,
): boolean => {
  try {
    // Parse the JSON file
    const parsedArray = JSON.parse(file).Histogram as Array<any>;

    if (parsedArray.length % 2 !== 0) {
      // "Data is not in correct format for histogram."
      return false;
    }

    // Transform the flat array into an array of pairs [string, number]
    const histo: Array<[string, number]> = [];
    for (let i = 0; i < parsedArray.length; i += 2) {
      histo.push([parsedArray[i], parsedArray[i + 1]]);
    }

    streamCallback(
      {
        buckets: histo,
        shotCount: shots,
      },
      "copilotResponseHistogram",
    );

    return true;
  } catch (e: any) {
    log.error("Error rendering results as histogram: ", e);
    return false;
  }
};

export const DownloadJobResults = async (
  jobId: string,
  conversation: CopilotConversation,
): Promise<string> => {
  const job = await GetJob(jobId, conversation);

  if (!job) {
    log.error("Failed to find the job.");
    return "Failed to find the job.";
  }

  if (job.status !== "Succeeded") {
    return "Job has not completed successfully.";
  }

  const workspace = await getConversationWorkspace(conversation);

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
      // log.info("Downloaded file: ", file);

      if (!tryRenderResults(file, job.shots, conversation.streamCallback)) {
        const doc = await vscode.workspace.openTextDocument({
          content: file,
          language: "json",
        });
        vscode.window.showTextDocument(doc);
        return "Results downloaded successfully as file.";
      } else {
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
};

export const GetProviders = async (
  conversation: CopilotConversation,
): Promise<Provider[]> => {
  const workspace = await getConversationWorkspace(conversation);
  return workspace?.providers ?? [];
};

export const GetTarget = async (
  targetId: string,
  conversation: CopilotConversation,
): Promise<Target | undefined> => {
  const providers = await GetProviders(conversation);
  for (const provider of providers) {
    const target = provider.targets.find((target) => target.id === targetId);
    if (target) {
      return target;
    }
  }
};

export const SubmitToTarget = async (
  jobName: string,
  targetId: string,
  numberOfShots: number,
  conversation: CopilotConversation,
): Promise<string> => {
  const target = await GetTarget(targetId, conversation);
  if (!target || target.currentAvailability !== "Available")
    return "Target not available.";

  const workspace = await getConversationWorkspace(conversation);

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
};

export const ToolCallSwitch = async (
  tool_name: string,
  args: any,
  copilotConversation: CopilotConversation,
): Promise<any> => {
  const content: any = {};

  log.info("Tool call name: ", tool_name);
  log.info("Tool call args: ", args);
  if (tool_name === "GetJob") {
    const job_id = args.job_id;
    const job = await GetJob(job_id, copilotConversation);
    content.job = job;
  } else if (tool_name === "GetJobs") {
    const recent_jobs = await GetJobs(copilotConversation);
    content.recent_jobs = recent_jobs;
  } else if (tool_name === "GetWorkspaces") {
    const workspace_ids = await GetWorkspaces();
    content.workspace_ids = workspace_ids;
  } else if (tool_name === "GetActiveWorkspace") {
    const active_workspace = await GetActiveWorkspace(copilotConversation);
    content.active_workspace = active_workspace;
  } else if (tool_name === "SetActiveWorkspace") {
    const workspace_id = args.workspace_id;
    const result = await SetActiveWorkspace(workspace_id, copilotConversation);
    content.result = result;
  } else if (tool_name === "DownloadJobResults") {
    const job_id = args.job_id;
    const download_result = await DownloadJobResults(
      job_id,
      copilotConversation,
    );
    content.download_result = download_result;
  } else if (tool_name === "GetProviders") {
    const providers = await GetProviders(copilotConversation);
    content.providers = providers;
  } else if (tool_name === "GetTarget") {
    const target_id = args.target_id;
    const target = await GetTarget(target_id, copilotConversation);
    content.target = target;
  } else if (tool_name === "SubmitToTarget") {
    const job_name = args.job_name;
    const target_id = args.target_id;
    const number_of_shots = args.number_of_shots;
    const submit_result = await SubmitToTarget(
      job_name,
      target_id,
      number_of_shots,
      copilotConversation,
    );
    content.submit_result = submit_result;
  }

  return content;
};
