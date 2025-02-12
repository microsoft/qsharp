// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import {
  ChatElement,
  CopilotUpdate,
  QuantumChatMessage,
  Status,
  ToolCall,
} from "./shared";
import { AzureQuantumChatBackend as AzureQuantumChatService } from "./azqChatService.js";
import { executeTool, ToolState } from "./tools.js";
import {
  getDefaultConfig,
  getServices,
  ServiceConfig,
  getShowDebugUiConfig,
} from "./config";
import { OpenAIChatService } from "./openAiChatService";
import { getRandomGuid } from "../utils";
import { EventType, sendTelemetryEvent, UserFlowStatus } from "../telemetry";
import { azqToolDefinitions, knownToolNameOrDefault } from "./azqTools";

export class Copilot {
  private service: IChatService;
  private messages: ChatElement[] = [];
  private workspaceState: ToolState = {};
  private serviceOptions: ServiceConfig[];
  private serviceConfigName: string;
  private conversationId: string = getRandomGuid();

  constructor(private onUpdate: CopilotUpdateHandler) {
    this.serviceOptions = getServices();
    this.serviceConfigName = getDefaultConfig().name;
    this.service = this.createService(this.serviceConfigName);
  }

  /**
   * Post a user message and wait for the assistant response.
   *
   * This essentially corresponds to submitting a user message through the UI.
   *
   * Updates will be posted via the `onUpdate` handler until the assistant
   * response is received, all the tool calls handled, and the chat is ready
   * for the next user message.
   */
  async postUserMessage(userMessage: string): Promise<void> {
    // Check the tip of the chat history to see if the last message was from the user.
    // If it was,
    //   - we may be in an error state (e.g. service error) or
    //   - there may be a double submission (should be mitigated by the UI).
    // Either way, remove the last message.
    if (
      this.messages.length > 0 &&
      this.messages[this.messages.length - 1].role === "user"
    ) {
      this.messages.pop();
    }
    this.messages.push({ role: "user", content: userMessage });

    await this.completeChat();
  }

  /**
   * Reinitialize the chat with a new history, optionally changing the
   * service backend.
   *
   * If the last message was from the user, an assistant response will
   * be requested as well.
   *
   * This is useful for when the user wants to retry sending a message,
   * start the chat over, or try with a different service backend
   * (useful for debugging).
   */
  async restartChat(history: ChatElement[], service?: string) {
    this.serviceConfigName = service ?? this.serviceConfigName;
    this.service = this.createService(this.serviceConfigName);
    this.conversationId = getRandomGuid();

    this.messages = history;
    this.sendFullUpdate({ status: "ready" });

    if (this.messages.length > 0) {
      const lastMessage = this.messages[this.messages.length - 1];
      if (lastMessage.role === "user") {
        await this.completeChat();
      }
    }
  }

  /**
   * Does a full "round" of chat completion requests. This may correspond
   * to multiple API requests.
   *
   * What this means is that if the assistant comes back with a
   * tool call request, we will execute the tool calls and make a request again,
   * and so on and so forth until the assistant provides a response
   * that doesn't require any tool calls.
   *
   * After a chat is "complete", the last message in the chat history
   * should be an assistant response.
   */
  private async completeChat() {
    const associationId = getRandomGuid();
    sendTelemetryEvent(
      EventType.ChatTurnStart,
      {
        conversationId: this.conversationId,
        associationId,
        serviceUrlHash: await this.service.getAnonymizedEnpdoint(),
      },
      {
        conversationMessages: this.messages.length,
      },
    );
    const startMs = performance.now();
    const allToolCalls: string[] = [];

    // Loop getting assistant responses and executing tool calls
    // until the assistant response doesn't require any tool calls.

    let result = await this.requestAssistantResponse();

    while (!result.done) {
      await this.executeToolCalls(result.toolCalls);

      // don't include arguments in telemetry
      allToolCalls.push(
        ...result.toolCalls.map((tc) => knownToolNameOrDefault(tc.name)),
      );

      result = await this.requestAssistantResponse();
    }

    sendTelemetryEvent(
      EventType.ChatTurnEnd,
      {
        associationId,
        flowStatus: result.success
          ? UserFlowStatus.Succeeded
          : UserFlowStatus.Failed,
        toolCalls: JSON.stringify(allToolCalls),
      },
      {
        timeToCompleteMs: performance.now() - startMs,
      },
    );
  }

  /**
   * Makes one API call to get an assistant response, communicating
   * status updates and deltas to the UI.
   */
  async requestAssistantResponse(): Promise<
    | {
        done: true;
        success: boolean;
      }
    | { done: false; toolCalls: ToolCall[] }
  > {
    this.sendFullUpdate({ status: "waitingAssistantResponse" });

    // "Widget" messages are visible to the client only and not
    // sent to the service.
    const messages = this.messages.filter((m) => m.role !== "widget");

    let content, toolCalls;

    try {
      ({ content, toolCalls } = await this.service.requestChatCompletion(
        messages,
        this.sendDelta.bind(this),
      ));
    } catch (e) {
      // Service request failed
      log.error("Chat request failed", e);
      this.sendStatusUpdateOnly({ status: "assistantConnectionError" });
      return { done: true, success: false };
    }

    this.messages.push({
      role: "assistant",
      content: cleanContent(content),
      ToolCalls: toolCalls,
    });

    if (content) {
      this.sendFullUpdate({ status: "ready" });
    }

    if (toolCalls) {
      return { done: false, toolCalls };
    }

    return { done: true, success: true };
  }

  /**
   * Execute tool calls using the local tool handlers.
   */
  async executeToolCalls(toolCalls: ToolCall[]) {
    for (const toolCall of toolCalls) {
      const statusString = azqToolDefinitions[toolCall.name].statusMessage;

      this.sendStatusUpdateOnly({
        status: "executingTool",
        toolStatus: statusString,
      });

      const toolResult = await executeTool(
        toolCall.name,
        toolCall.arguments,
        this.workspaceState,
      );

      // If the tool response contains a widget,
      // add that to the chat history as a special "widget" message.
      if ("widgetData" in toolResult && toolResult.widgetData) {
        this.messages.push({
          role: "widget",
          widgetData: toolResult.widgetData,
        });

        // Don't include the widget data in the tool response
        delete toolResult.widgetData;
      }

      this.messages.push({
        role: "tool",
        content: JSON.stringify(toolResult),
        toolCallId: toolCall.id,
      });
      this.sendFullUpdate({ status: "ready" });
    }
  }

  /**
   * Notify listener (the UI) of a state update. Includes the full state.
   */
  private sendFullUpdate(status: Status) {
    this.onUpdate({
      kind: "updateChat",
      payload: {
        history: this.messages,
        status,
        debugUi: getShowDebugUiConfig()
          ? {
              show: true,
              service: this.serviceConfigName!,
              serviceOptions: this.serviceOptions.map((s) => s.name),
            }
          : { show: false },
      },
    });
  }

  /**
   * Notify listener of a change in the status only.
   */
  private sendStatusUpdateOnly(status: Status) {
    this.onUpdate({
      kind: "updateStatus",
      payload: { status },
    });
  }

  /**
   * Notify listener of a message delta.
   *
   * Deltas are just chunks of a streaming assistant response.
   * After deltas have been sent, the UI can expect to receive
   * an update with the full assistant response.
   */
  private sendDelta(delta: string) {
    this.onUpdate({
      kind: "appendDelta",
      payload: {
        delta: cleanContent(delta),
        status: { status: "waitingAssistantResponse" },
      },
    });
  }

  createService(serviceName: string): IChatService {
    const service = this.serviceOptions.find((s) => s.name === serviceName);
    if (!service) {
      log.error(`Service ${serviceName} not found in configuration`);
      throw new Error(`Service ${serviceName} not found in configuration`);
    }

    switch (service.type) {
      case "AzureQuantum":
        return new AzureQuantumChatService(service);
      case "OpenAI":
        return new OpenAIChatService(service);
      default:
        throw new Error('Unknown service type, try "OpenAI" or "AzureQuantum"');
    }
  }
}

export type CopilotUpdateHandler = (event: CopilotUpdate) => void;

/**
 * An abstraction for the various service backend options.
 */
export interface IChatService {
  /**
   * Post one chat request.
   */
  requestChatCompletion(
    messages: QuantumChatMessage[],
    onDelta: (delta: string) => void,
  ): Promise<{
    content?: string;
    toolCalls?: ToolCall[];
  }>;

  /**
   * Get a unique, anonymous, endpoint identifier,
   * used in telemetry to differentiate various service APIs we may use.
   */
  getAnonymizedEnpdoint(): Promise<string>;
}

function cleanContent(content: string | undefined) {
  // Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
  let cleanedResponse = content;
  if (cleanedResponse) {
    cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
    cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");
  }
  return cleanedResponse || "";
}
