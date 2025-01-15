import { ChatCompletionTool } from "openai/resources/index.mjs";
import { Job, WorkspaceConnection } from "../azure/treeView";
import { getConversationWorkspace } from "./copilotTools";
import { ConversationState } from "./copilot";

const jobLimit = 10;
const jobLimitDays = 14;

export const getJobsToolDescription: ChatCompletionTool = {
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
};

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

type GetJobsResult = {
  recentJobs: MinimizedJob[];
};

export async function handleGetJobs(
  conversationState: ConversationState,
): Promise<GetJobsResult> {
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
    recentJobs,
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
  outputDataUri?: string;
  count: number;
  shots: number;
  creationTime: string;
  beginExecutionTime?: string;
  endExecutionTime?: string;
  cancellationTime?: string;
  costEstimate?: any;
  errorData?: { code: string; message: string };
};
