/* eslint-disable @typescript-eslint/no-unused-vars */
import OpenAI from "openai";
//import * as readline from "readline";
import * as vscode from "vscode";
import { log } from "qsharp-lang";
import {
  ChatCompletion,
  ChatCompletionMessageParam,
  ChatCompletionMessageToolCall,
  ChatCompletionTool,
} from "openai/resources/chat/completions";
import {
  Job,
  Provider,
  Target,
  WorkspaceConnection,
  WorkspaceTreeProvider,
} from "./azure/treeView.js";
import { QuantumUris } from "./azure/networkRequests.js";
import { getTokenForWorkspace } from "./azure/auth.js";
import { getJobFiles, submitJob } from "./azure/workspaceActions.js";
import { supportsAdaptive } from "./azure/providerProperties.js";
import { getQirForActiveWindow, getQirForVisibleQs } from "./qirGeneration.js";

// Don't check in a filled in API key
const openai = new OpenAI({
  apiKey: "",
});

// Create a readline interface to read user input

// const rl = readline.createInterface({
//   input: process.stdin,
//   output: process.stdout,
// });

// async function askQuestion(query: string): Promise<string> {
//   return new Promise((resolve) => rl.question(query, resolve));
// }

// Define mock data

enum JobStatus {
  InProgress = "In Progress",
  Completed = "Completed",
  Failed = "Failed",
  NotFound = "Not Found",
}

const mockJobs: { [id: string]: JobStatus } = {
  "4": JobStatus.Completed,
  "6": JobStatus.InProgress,
  "7": JobStatus.InProgress,
  "9": JobStatus.Completed,
  "11": JobStatus.Failed,
  "13": JobStatus.InProgress,
  "14": JobStatus.Completed,
  "16": JobStatus.Completed,
  "17": JobStatus.Failed,
  "23": JobStatus.InProgress,
};

const mockJobs2: Job[] = [
  {
    id: "1",
    name: "First Job",
    target: "Quantum Hardware",
    status: "Succeeded",
    creationTime: "8/19/2024, 9:19:24 AM",
    beginExecutionTime: "8/19/2024, 9:19:50 AM",
    endExecutionTime: "8/19/2024, 9:21:04 AM",
  },
  {
    id: "2",
    name: "Second Job",
    target: "Quantum Hardware",
    status: "Failed",
    creationTime: "8/19/2024, 9:21:54 AM",
    beginExecutionTime: "8/19/2024, 9:22:11 AM",
    endExecutionTime: "8/19/2024, 9:22:34 AM",
  },
  {
    id: "3",
    name: "Third Job",
    target: "Quantum Hardware",
    status: "Succeeded",
    creationTime: "8/20/2024, 9:19:24 AM",
    beginExecutionTime: "8/20/2024, 9:19:50 AM",
    endExecutionTime: "8/20/2024, 9:21:04 AM",
  },
  {
    id: "4",
    name: "Other Job",
    target: "Quantum Hardware",
    status: "Succeeded",
    creationTime: "8/21/2024, 9:19:24 AM",
    beginExecutionTime: "8/21/2024, 9:19:50 AM",
    endExecutionTime: "8/21/2024, 9:21:04 AM",
  },
];

// Define the tools and system prompt that the model can use

// const GetJobStatus = async (jobId: string): Promise<string> => {
//   return jobId in mockJobs ? mockJobs[jobId] : JobStatus.NotFound;
// };

// const GetRecentJobs = async (): Promise<string> => {
//   let jobs = "";
//   for (const [job_id, _] of Object.entries(mockJobs)) {
//     await GetJobStatus(job_id).then((status) => {
//       jobs += `\nJob ${job_id}: ${status}`;
//     });
//   }
//   return jobs;
// };

const tools: ChatCompletionTool[] = [
  // {
  //   type: "function",
  //   function: {
  //     name: "GetJobStatus",
  //     description:
  //       "Get the job status for a customer's job. Call this whenever you need to know the job status, for example when a customer asks 'What is the status of my job?'",
  //     strict: true,
  //     parameters: {
  //       type: "object",
  //       properties: {
  //         job_id: {
  //           type: "string",
  //           description: "The customer's job ID.",
  //         },
  //       },
  //       required: ["job_id"],
  //       additionalProperties: false,
  //     },
  //   },
  // },
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
      name: "DownloadJobResults",
      description:
        "Download and display the results from a customer's job. Call this when you need to display or download the results for a job, for example when a customer asks 'What are the results of my last job?'",
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
        "Submit the current Q# program to Azure Quantum with the provided hardware target. Call this when you need to submit or run a Q# program with Azure Quantum, for example when a customer asks 'Can you submit this program to Azure?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          target_id: {
            type: "string",
            description: "The ID or name of the target to submit the job to.",
          },
        },
        required: ["target_id"],
        additionalProperties: false,
      },
    },
  },
];

const systemMessage: ChatCompletionMessageParam = {
  role: "system",
  content:
    "You are a helpful customer support assistant. Use the supplied tools to assist the user. " +
    'Do not provide information about jobs whose status is "Not Found", unless the user specifically asks for the job by it\'s id. ' +
    "Do not provide container URI links from jobs to the user. ",
};

// Azure stuff

const job_limit = 10;

const GetJobs = async (): Promise<Job[]> => {
  //return mockJobs2;
  const workspace = await getPrimaryWorkspace();
  if (workspace) {
    const jobs = workspace.jobs;

    const limited_jobs =
      jobs.length > job_limit ? jobs.slice(0, job_limit) : jobs;

    return limited_jobs;
  } else {
    return [];
  }
};

// For now, let's just grab the first workspace
const getPrimaryWorkspace = async (): Promise<
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

const GetWorkspaces = async (): Promise<string[]> => {
  const tree = WorkspaceTreeProvider.instance;
  return tree.getWorkspaceIds();
};

const GetJob = async (jobId: string): Promise<Job | undefined> => {
  const jobs = await GetJobs();
  return jobs.find((job) => job.id === jobId);
};

const GetJobStatus = async (jobId: string): Promise<string> => {
  const job = await GetJob(jobId);
  if (job) {
    return job.status;
  } else {
    return "Not Found";
  }
};

const tryRenderResults = (_file: string): boolean => {
  // Not implemented yet
  return false;
  // Test string for rendering histogram
  const file = '{"Histogram":["[0, 0, 0]",0.52,"[1, 1, 1]",0.48]}';

  if (file.startsWith("```widget\n")) {
    if (file.includes("Histogram")) {
      // Render histogram
      vscode.window.showInformationMessage("Rendering histogram");
      return true;
    } else if (file.includes("Results")) {
      // Render results table
      vscode.window.showInformationMessage("Rendering results table");
      return true;
    }
  }
  return false;
};

const DownloadJobResults = async (jobId: string): Promise<string> => {
  const job = await GetJob(jobId);

  if (!job) {
    log.error("Failed to find the job.");
    return "Failed to find the job.";
  }

  if (job.status !== "Succeeded") {
    return "Job has not completed successfully.";
  }

  const workspace = await getPrimaryWorkspace(); // Note that we are getting the primary workspace again here

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
      log.info("Downloaded file: ", file);

      if (!tryRenderResults(file)) {
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

const GetProviders = async (): Promise<Provider[]> => {
  const workspace = await getPrimaryWorkspace();
  return workspace?.providers ?? [];
};

const GetTarget = async (targetId: string): Promise<Target | undefined> => {
  const providers = await GetProviders();
  for (const provider of providers) {
    const target = provider.targets.find((target) => target.id === targetId);
    if (target) {
      return target;
    }
  }
};

const SubmitToTarget = async (targetId: string): Promise<string> => {
  // // Could be run via the treeItem icon or the menu command.
  // const treeItem = arg || currentTreeItem;
  // if (treeItem?.type !== "target") return;

  // const target = treeItem.itemData as Target;

  const target = await GetTarget(targetId);
  if (!target || target.currentAvailability !== "Available")
    return "Target not available.";

  const workspace = await getPrimaryWorkspace(); // Note that we are getting the primary workspace again here

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

    const jobId = await submitJob(
      token,
      quantumUris,
      qir,
      providerId,
      target.id,
    );
    // if (jobId) {
    //   // The job submitted fine. Refresh the workspace until it shows up
    //   // and all jobs are finished. Don't await on this, just let it run
    //   startRefreshCycle(workspaceTreeProvider, treeItem.workspace, jobId);
    // }
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

export type CopilotStreamCallback = (mdFragment: string, done: boolean) => void;

export class Copilot {
  messages: ChatCompletionMessageParam[] = [];

  constructor() {
    this.messages.push(systemMessage);
  }

  // OpenAI handling functions

  converseWithOpenAI = async (displayResponse: string[]) => {
    //console.debug("Sent messages: %o", messages);
    const response = await openai.chat.completions.create({
      model: "gpt-4o-mini",
      messages: this.messages,
      tools: tools,
    });

    //console.debug("Response: %o", response);
    await this.handleResponse(response, displayResponse);
  };

  handleResponse = async (
    response: ChatCompletion,
    displayResponse: string[],
  ) => {
    this.messages.push(response.choices[0].message);

    // Check if the conversation was too long for the context window
    if (response.choices[0].finish_reason === "length") {
      // Handle the error as needed, e.g., by truncating the conversation or asking for clarification
      this.handleLengthError(response);
    }

    // Check if the model's output included copyright material (or similar)
    else if (response.choices[0].finish_reason === "content_filter") {
      // Handle the error as needed, e.g., by modifying the request or notifying the user
      this.handleContentFilterError(response);
    }

    // Check if the model has made a tool_call.
    else if (response.choices[0].finish_reason === "tool_calls") {
      // Handle tool call
      await this.handleToolCalls(response, displayResponse);
    }

    // Else finish_reason is "stop", in which case the model was just responding directly to the user
    else if (response.choices[0].finish_reason === "stop") {
      // Handle the normal stop case
      this.handleNormalResponse(response, displayResponse);
    }

    // Catch any other case, this is unexpected
    else {
      // Handle unexpected cases as needed
      this.handleUnexpectedCase(response);
    }
  };

  handleToolCalls = async (
    response: ChatCompletion,
    displayResponse: string[],
  ) => {
    if (response.choices[0].message.tool_calls) {
      for (const toolCall of response.choices[0].message.tool_calls) {
        const content = await this.handleSingleToolCall(toolCall);
        // Create a message containing the result of the function call
        const function_call_result_message: ChatCompletionMessageParam = {
          role: "tool",
          content: JSON.stringify(content),
          tool_call_id: toolCall.id,
        };
        this.messages.push(function_call_result_message);
      }

      await this.converseWithOpenAI(displayResponse);
    }
  };

  handleSingleToolCall = async (toolCall: ChatCompletionMessageToolCall) => {
    const args = JSON.parse(toolCall.function.arguments);

    const content: any = {};

    if (toolCall.function.name === "GetJobStatus") {
      const job_id = args.job_id;
      //console.log("Tool Call: GetJobStatus");
      const status = await GetJobStatus(job_id);
      content.status = status;
    } else if (toolCall.function.name === "GetJob") {
      const job_id = args.job_id;
      const job = await GetJob(job_id);
      content.job = job;
    } else if (toolCall.function.name === "GetJobs") {
      //console.log("Tool Call: GetRecentJobs");
      const recent_jobs = await GetJobs();
      content.recent_jobs = recent_jobs;
    } else if (toolCall.function.name === "GetWorkspaces") {
      const workspace_ids = await GetWorkspaces();
      content.workspace_ids = workspace_ids;
    } else if (toolCall.function.name === "DownloadJobResults") {
      const job_id = args.job_id;
      const download_result = await DownloadJobResults(job_id);
      content.download_result = download_result;
    } else if (toolCall.function.name === "GetProviders") {
      const providers = await GetProviders();
      content.providers = providers;
    } else if (toolCall.function.name === "GetTarget") {
      const target_id = args.target_id;
      const target = await GetTarget(target_id);
      content.target = target;
    } else if (toolCall.function.name === "SubmitToTarget") {
      const target_id = args.target_id;
      const submit_result = await SubmitToTarget(target_id);
      content.submit_result = submit_result;
    }

    return content;
  };

  handleLengthError = (response: ChatCompletion) => {
    console.log("Error: The conversation was too long for the context window.");
  };

  handleContentFilterError = (response: ChatCompletion) => {
    console.log("Error: The content was filtered due to policy violations.");
  };

  handleNormalResponse = (response: ChatCompletion, chatResponse: string[]) => {
    chatResponse.push(response.choices[0].message.content!);
  };

  handleUnexpectedCase = (response: ChatCompletion) => {
    console.log("Unexpected response: %o", response.choices[0]);
  };

  async makeChatRequest(
    question: string,
    streamCallback: CopilotStreamCallback,
  ) {
    this.messages.push({
      role: "user",
      content: question,
    });

    const displayResponse: string[] = [];
    await this.converseWithOpenAI(displayResponse);

    for (const response of displayResponse) {
      streamCallback(response, false);
    }
    streamCallback("", true);
  }
}

// export async function foo(
//   question: string,
//   streamCallback: CopilotStreamCallback,
// ) {
//   const messages: ChatCompletionMessageParam[] = [
//     {
//       role: "user",
//       content: question,
//     },
//   ];

//   const chatResponse: string[] = [];
//   await converseWithOpenAI(messages, chatResponse);

//   for (const response of chatResponse) {
//     streamCallback(response, false);
//   }
//   streamCallback("", true);

//   //return chatResponse.join("\n");
// }

// async function main() {
//   const userQuestion = await askQuestion(
//     "Please enter a question for the chatbot: "
//   );
//   let response = await foo(userQuestion);

//   console.log(response);

//   rl.close(); // Ensure the readline interface is closed
// }

// main().catch(console.error);
