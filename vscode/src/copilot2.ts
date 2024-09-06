import OpenAI from "openai";
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
import {
  getJobFiles,
  submitJobWithNameAndShots,
} from "./azure/workspaceActions.js";
import { supportsAdaptive } from "./azure/providerProperties.js";
import { getQirForVisibleQs } from "./qirGeneration.js";

// Don't check in a filled in API key
const openai = new OpenAI({
  apiKey: "",
});

// Define the tools and system prompt that the model can use

const tools: ChatCompletionTool[] = [
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

// ToDo: For now, let's just grab the first workspace
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

const tryRenderResults = (
  file: string,
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
        shotCount: 100, // ToDo: Where are the actual shot counts stored?
      },
      "copilotResponseHistogram",
    );

    return true;
  } catch (e: any) {
    log.error("Error rendering results as histogram: ", e);
    return false;
  }
};

const DownloadJobResults = async (
  jobId: string,
  streamCallback: CopilotStreamCallback,
): Promise<string> => {
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
      // log.info("Downloaded file: ", file);

      if (!tryRenderResults(file, streamCallback)) {
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

const SubmitToTarget = async (
  jobName: string,
  targetId: string,
  numberOfShots: number,
): Promise<string> => {
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

export type CopilotStreamCallback = (
  msgPayload: object,
  msgCommand:
    | "copilotResponse"
    | "copilotResponseDone"
    | "copilotResponseHistogram",
) => void;

export class Copilot {
  messages: ChatCompletionMessageParam[] = [];
  streamCallback: CopilotStreamCallback;

  constructor(streamCallback: CopilotStreamCallback) {
    this.messages.push(systemMessage);
    this.streamCallback = streamCallback;
  }

  // OpenAI handling functions

  converseWithOpenAI = async () => {
    // log.info("Sent messages: %o", this.messages);
    const response = await openai.chat.completions.create({
      model: "gpt-4o-mini",
      messages: this.messages,
      tools: tools,
    });
    // log.info("Response: %o", response);
    await this.handleResponse(response);
  };

  handleResponse = async (response: ChatCompletion) => {
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
      await this.handleToolCalls(response);
    }

    // Else finish_reason is "stop", in which case the model was just responding directly to the user
    else if (response.choices[0].finish_reason === "stop") {
      // Handle the normal stop case
      this.handleNormalResponse(response);
    }

    // Catch any other case, this is unexpected
    else {
      // Handle unexpected cases as needed
      this.handleUnexpectedCase(response);
    }
  };

  handleToolCalls = async (response: ChatCompletion) => {
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

      await this.converseWithOpenAI();
    }
  };

  handleSingleToolCall = async (toolCall: ChatCompletionMessageToolCall) => {
    // log.info("Tool call: %o", toolCall);

    const args = JSON.parse(toolCall.function.arguments);

    const content: any = {};

    if (toolCall.function.name === "GetJob") {
      const job_id = args.job_id;
      const job = await GetJob(job_id);
      content.job = job;
    } else if (toolCall.function.name === "GetJobs") {
      const recent_jobs = await GetJobs();
      content.recent_jobs = recent_jobs;
    } else if (toolCall.function.name === "GetWorkspaces") {
      const workspace_ids = await GetWorkspaces();
      content.workspace_ids = workspace_ids;
    } else if (toolCall.function.name === "DownloadJobResults") {
      const job_id = args.job_id;
      const download_result = await DownloadJobResults(
        job_id,
        this.streamCallback,
      );
      content.download_result = download_result;
    } else if (toolCall.function.name === "GetProviders") {
      const providers = await GetProviders();
      content.providers = providers;
    } else if (toolCall.function.name === "GetTarget") {
      const target_id = args.target_id;
      const target = await GetTarget(target_id);
      content.target = target;
    } else if (toolCall.function.name === "SubmitToTarget") {
      const job_name = args.job_name;
      const target_id = args.target_id;
      const number_of_shots = args.number_of_shots;
      const submit_result = await SubmitToTarget(
        job_name,
        target_id,
        number_of_shots,
      );
      content.submit_result = submit_result;
    }

    return content;
  };

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  handleLengthError = (_response: ChatCompletion) => {
    log.error("Error: The conversation was too long for the context window.");
  };

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  handleContentFilterError = (_response: ChatCompletion) => {
    log.error("Error: The content was filtered due to policy violations.");
  };

  handleNormalResponse = (response: ChatCompletion) => {
    // log.info("printing response: %o", response.choices[0].message.content!);
    this.streamCallback(
      {
        response: response.choices[0].message.content!,
      },
      "copilotResponse",
    );
  };

  handleUnexpectedCase = (response: ChatCompletion) => {
    log.error("Unexpected response: %o", response.choices[0]);
  };

  async makeChatRequest(question: string) {
    this.messages.push({
      role: "user",
      content: question,
    });

    await this.converseWithOpenAI();
    this.streamCallback({}, "copilotResponseDone");
  }
}
