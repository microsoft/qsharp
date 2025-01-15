import OpenAI from "openai";
import { log } from "qsharp-lang";
import { ConversationState } from "./copilot/azqCopilot.js";
import { CopilotEventHandler, ICopilot } from "./copilot/copilot.js";
import { executeTool } from "./copilot/copilotTools.js";
import { apiKey } from "./copilotApiKey.js";

// Don't check in a filled in API key
const openai = new OpenAI({
  apiKey,
});

// Define the tools and system prompt that the model can use

const tools: OpenAI.ChatCompletionTool[] = [
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

const systemMessage: OpenAI.ChatCompletionMessageParam = {
  role: "system",
  content:
    "You are a helpful customer support assistant. Use the supplied tools to assist the user. " +
    'Do not provide information about jobs whose status is "Not Found", unless the user specifically asks for the job by it\'s id. ' +
    "Do not provide container URI links from jobs to the user. ",
};

export class OpenAICopilot implements ICopilot {
  messages: OpenAI.ChatCompletionMessageParam[] = [];
  streamCallback: CopilotEventHandler;
  conversationState: ConversationState;

  constructor(streamCallback: CopilotEventHandler) {
    this.conversationState = { sendMessage: streamCallback };
    this.messages.push(systemMessage);
    this.streamCallback = streamCallback;
  }

  // OpenAI handling functions

  async converse(question: string): Promise<void> {
    this.messages.push({ role: "user", content: question });

    const { content, toolCalls } = await this.converseWithCopilot();
    await this.handleFullResponse(content, toolCalls);

    this.conversationState.sendMessage({
      kind: "copilotResponseDone",
      payload: { history: this.messages },
    });
  }

  async handleFullResponse(
    content?: string,
    toolCalls?: OpenAI.ChatCompletionMessageToolCall[],
  ): Promise<void> {
    this.messages.push({
      role: "assistant",
      content: content || "",
      tool_calls: toolCalls,
    });
    if (content) {
      this.conversationState.sendMessage({
        kind: "copilotResponse",
        payload: { response: content },
      });
    }
    if (toolCalls) {
      await this.handleToolCalls(toolCalls);
      {
        const { content, toolCalls } = await this.converseWithCopilot();
        await this.handleFullResponse(content, toolCalls);
      }
    }
  }

  async converseWithCopilot(): Promise<{
    content?: string;
    toolCalls?: OpenAI.ChatCompletionMessageToolCall[];
  }> {
    const payload = {
      model: "gpt-4o-mini",
      messages: this.messages,
      tools: tools,
    };

    log.debug(
      `making request to OpenAI, payload:\n${JSON.stringify(payload, undefined, 2)}`,
    );

    try {
      const response = await openai.chat.completions.create(payload);
      const contentInResponse: string | undefined =
        response.choices[0].message.content ?? undefined;
      const toolCallsInResponse:
        | OpenAI.ChatCompletionMessageToolCall[]
        | undefined = response.choices[0].message.tool_calls ?? undefined;

      return { content: contentInResponse, toolCalls: toolCallsInResponse };
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
  }

  async handleToolCalls(toolCalls: OpenAI.ChatCompletionMessageToolCall[]) {
    for (const toolCall of toolCalls) {
      this.conversationState.sendMessage({
        kind: "copilotToolCall",
        payload: { toolName: toolCall.function.name },
      });
      const content = await executeTool(
        toolCall.function.name,
        JSON.parse(toolCall.function.arguments),
        this.conversationState,
      );
      // Create a message containing the result of the function call
      const function_call_result_message: OpenAI.ChatCompletionToolMessageParam =
        {
          role: "tool",
          content: JSON.stringify(content),
          tool_call_id: toolCall.id,
        };
      this.messages.push(function_call_result_message);
    }
  }
}
