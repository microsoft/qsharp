import OpenAI from "openai";
import { log } from "qsharp-lang";
import {
  ChatCompletion,
  ChatCompletionMessageParam,
  ChatCompletionMessageToolCall,
} from "openai/resources/chat/completions";
import {
  CopilotStreamCallback,
  ToolCallSwitch,
  CopilotToolsDescriptions,
} from "./copilotTools.js";

// Don't check in a filled in API key
const openai = new OpenAI({
  apiKey: "",
});

const systemMessage: ChatCompletionMessageParam = {
  role: "system",
  content:
    "You are a helpful customer support assistant. Use the supplied tools to assist the user. " +
    'Do not provide information about jobs whose status is "Not Found", unless the user specifically asks for the job by it\'s id. ' +
    "Do not provide container URI links from jobs to the user. ",
};

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
      tools: CopilotToolsDescriptions,
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

    return ToolCallSwitch(toolCall.function.name, args, this.streamCallback);
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
