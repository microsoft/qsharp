import OpenAI from "openai";
import { log } from "qsharp-lang";
import { IChatService } from "./copilot.js";
// Don't check in an API key
import { azqToolDefinitions } from "./azqTools.js";
import { apiKey, systemPrompt } from "./secrets.js";
import { QuantumChatMessage, ToolCall } from "./shared.js";

const openai = new OpenAI({ apiKey });

export class OpenAiChatBackend implements IChatService {
  async requestChatCompletion(
    messages: QuantumChatMessage[],
    handleResponseDelta: (delta: string) => void,
  ): Promise<{ content?: string; toolCalls?: ToolCall[] }> {
    const payload = {
      stream: true,
      model: "gpt-4o-mini",
      messages: [
        systemPrompt,
        ...messages.map((m) => {
          switch (m.role) {
            case "user":
              return {
                role: m.role,
                content: m.content,
              } as OpenAI.ChatCompletionUserMessageParam;
            case "assistant":
              return {
                role: m.role,
                content: m.content,
                tool_calls: m.ToolCalls?.map((tc) => {
                  return {
                    type: "function",
                    id: tc.id,
                    function: {
                      name: tc.name,
                      arguments: tc.arguments,
                    },
                  } as OpenAI.ChatCompletionMessageToolCall;
                }),
              } as OpenAI.ChatCompletionAssistantMessageParam;
            case "tool":
              return {
                role: m.role,
                content: m.content,
                tool_call_id: m.toolCallId,
              } as OpenAI.ChatCompletionToolMessageParam;
          }
        }),
      ],
      tools: tools,
    };

    try {
      const stream = await openai.chat.completions.create(
        payload as OpenAI.ChatCompletionCreateParamsStreaming,
      );

      let content = "";
      let toolCalls:
        | OpenAI.ChatCompletionChunk.Choice.Delta.ToolCall[]
        | undefined = undefined;

      for await (const response of stream) {
        const choice = response.choices[0];
        const contentDelta = choice.delta.content;
        if (contentDelta) {
          handleResponseDelta(contentDelta);
          content += contentDelta;
        }

        if (choice.delta.tool_calls && choice.delta.tool_calls.length > 0) {
          for (const toolCall of choice.delta.tool_calls) {
            // This handling is a bit weird because the tool calls are also streamed in chunks.
            // A single tool call can be split across multiple messages.
            // As far as we can tell, the `arguments` string is sent in chunks but the
            // other fields are not.
            toolCalls = toolCalls || [];
            let entry = toolCalls[toolCall.index];
            if (!entry) {
              entry = toolCall;
              toolCalls[toolCall.index] = entry;
            }

            if (toolCall.id) {
              entry.id = toolCall.id;
            }

            if (toolCall.type) {
              entry.type = toolCall.type;
            }

            if (toolCall.function) {
              entry.function = entry.function || {};
              if (toolCall.function.name) {
                entry.function.name = toolCall.function.name;
              }

              if (toolCall.function.arguments) {
                entry.function.arguments = entry.function.arguments
                  ? entry.function.arguments + toolCall.function.arguments
                  : toolCall.function.arguments;
              }
            }
          }
        }
      }

      return {
        content,
        toolCalls: toolCalls
          ?.filter(
            (tc) =>
              tc.id && tc.function && tc.function.name && tc.function.arguments,
          )
          .map((tc) => {
            return {
              id: tc.id!,
              name: tc.function!.name!,
              arguments: tc.function!.arguments!,
            };
          }),
      };
    } catch (error) {
      log.error("ChatAPI fetch failed with error: ", error);
      throw error;
    }
  }
}

/**
 * The tool definitions to be used by the OpenAI chat backend.
 *
 * These need to match our tool handlers, which in turn need to match
 * the tools that are defined in the Azure Quantum service, to ensure
 * these two service backends are interchangeable.
 */
const tools: OpenAI.ChatCompletionTool[] = [
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
      name: "ConnectToWorkspace",
      description:
        "Starts the UI flow to connect to an existing Azure Quantum Workspace. Call this when the customer does not have an active workspace, and agrees to connect to a workspace.",
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
      name: "GetJobs",
      description:
        "Get a list of recent jobs that have been run by the customer, along with their statuses, in the currently active workspace. Call this when you need to know what jobs have been run recently or need a history of jobs run, for example when a customer asks 'What are my recent jobs?'",
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
      name: "GetJob",
      description:
        "Get the job information for a customer's job given its id. Call this whenever you need to know information about a specific job, for example when a customer asks 'What is the status of my job?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          job_id: {
            type: "string",
            description: "Job's unique identifier.",
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
        "Get the target information for a specified target given its id. Call this whenever you need to know information about a specific target, for example when a customer asks 'What is the status of this target?'",
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
  {
    type: "function",
    function: {
      name: "DownloadJobResults",
      description:
        "Download and display the results from a customer's job given its id. Call this when you need to download or display as a histogram the results for a job, for example when a customer asks 'What are the results of my last job?' or 'Can you show me the histogram for this job?'",
      strict: true,
      parameters: {
        type: "object",
        properties: {
          job_id: {
            type: "string",
            description: "Job's unique identifier.",
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
      name: "GetActiveWorkspace",
      description:
        "Get the id of the active workspace for this conversation. Call this when you need to know what workspace is the active workspace being used in the context of the current conversation, for example when a customer asks 'What is the workspace that's being used?'",
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
        "Set the active workspace for this conversation by id. Call this when you need to set what workspace is the active workspace being used in the context of the current conversation, for example when a customer says 'Please use this workspace for my requests.'",
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
];

// Sanity check the tool definitions
for (const key in azqToolDefinitions) {
  const tool = tools.find((t) => t.function.name === key);
  if (!tool) {
    log.error(`Tool definition for ${key} not found in OpenAI tools.`);
  }
}
for (const tool of tools) {
  const handler = azqToolDefinitions[tool.function.name];
  if (!handler) {
    log.error(`Tool handler for ${tool.function.name} not found.`);
  }
}
