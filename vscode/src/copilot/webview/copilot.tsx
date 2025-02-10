// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

// Use esbuild to bundle and copy the CSS files to the output directory.
import "highlight.js/styles/default.css";
import "modern-normalize/modern-normalize.css";
import { render } from "preact";
import {
  ChatElement,
  CopilotCommand,
  CopilotUpdate,
  HistogramData,
  Status,
} from "../shared";
import "./copilot.css";
// Set up the Markdown renderer with KaTeX support
import mk from "@vscode/markdown-it-katex";
import hljs from "highlight.js";
import markdownIt from "markdown-it";
import { useEffect, useRef } from "preact/hooks";
import { Histogram, Markdown, setRenderer } from "qsharp-lang/ux";
import {
  RetryButton,
  ServiceSelector,
  ShowPayload,
  ToolMessage,
} from "./debugUi";
import hlsjQsharp from "./hlsj-qsharp";

/**
 * Enables UI elements that aren't included in the product
 * but are helpful for debugging service interactions.
 */
const enableDebugUi = true;

const vscodeApi = acquireVsCodeApi();

hljs.registerLanguage("qsharp", hlsjQsharp);
const md = markdownIt("commonmark", {
  highlight(str, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value;
      } catch (__) {
        console.error("Failed to highlight code block", __);
      }
    }
    return "";
  },
});
md.use(mk as any, {
  enableMathBlockInHtml: true,
  enableMathInlineInHtml: true,
});
setRenderer((input: string) => md.render(input));

function App({ model }: { model: ChatViewModel }) {
  return (
    <div className="app">
      <div className="chat">
        {model.history.length > 0 ? (
          <ChatHistory model={model} />
        ) : (
          <Markdown
            markdown={`# Azure Quantum Copilot

Welcome! I'm here to assist you with your Azure Quantum workspace and help you explore Q# programming.

Try:

- "Can you submit this Q# program to Azure Quantum for execution?"
- "What are the currently available hardware providers for my workspace?"
- "Can you show the results from my last job?"

Azure Quantum Copilot is powered by AI, so mistakes are possible. Review the output carefully before use.`}
          ></Markdown>
        )}
        <StatusIndicator status={model.status} />
        <InputBox
          disable={
            model.status.status !== "ready" &&
            model.status.status !== "assistantConnectionError"
          }
        />
      </div>
      {enableDebugUi ? (
        <>
          <ShowPayload
            history={model.history}
            service={model.service}
            serviceOptions={model.serviceOptions}
            restartChat={restartChat}
          />
          <ServiceSelector
            service={model.service}
            serviceOptions={model.serviceOptions}
            restartChat={restartChat}
          />
        </>
      ) : null}
    </div>
  );
}

/**
 * The full chat history that appears above the input box.
 */
function ChatHistory({ model }: { model: ChatViewModel }) {
  const elements = [];

  for (let i = 0; i < model.history.length; i++) {
    const message = model.history[i];

    switch (message.role) {
      case "assistant":
        // There may not be content if the response was just a tool call.
        if (message.content) {
          elements.push(<AssistantMessage content={message.content} />);
        }
        break;
      case "user":
        elements.push(<UserMessage content={message.content} />);
        if (enableDebugUi) {
          elements.push(
            <RetryButton
              history={model.history.slice(0, i + 1)}
              serviceOptions={model.serviceOptions}
              selectedService={model.service}
              restartChat={restartChat}
            ></RetryButton>,
          );
        }

        break;
      case "tool":
        if (enableDebugUi) {
          elements.push(
            <ToolMessage
              id={message.toolCallId}
              content={message.content}
              history={model.history}
            />,
          );
        }
        break;
      case "widget":
        elements.push(<Widget histogram={message.widgetData} />);
        break;
    }
  }

  // Append any partially completed response
  if (model.inProgressResponse.length > 0) {
    elements.push(<AssistantMessage content={model.inProgressResponse} />);
  }

  return <>{elements}</>;
}

/**
 * The status indicator just above the input box.
 */
function StatusIndicator({ status }: { status: Status }) {
  return (
    <div className="status-indicator">
      {status.status === "waitingAssistantResponse"
        ? "🕒"
        : status.status === "executingTool"
          ? "🕒 " + status.toolName
          : status.status === "assistantConnectionError"
            ? "There was an error communicating with Azure Quantum Copilot. Please check your Internet connection and try again."
            : ""}
    </div>
  );
}

/**
 * The input box for the user to type in.
 */
function InputBox(props: { disable: boolean }) {
  const textRef = useRef<HTMLTextAreaElement>(null);
  const hrRef = useRef<HTMLHRElement>(null);

  // Always bring the input box into view and focus
  // when the component is rendered.
  useEffect(() => {
    hrRef.current?.scrollIntoView(false);
    textRef.current?.focus();
  });

  function submit() {
    if (
      textRef.current &&
      textRef.current.value?.trim().length > 0 &&
      !props.disable
    ) {
      submitUserMessage(textRef.current.value);
      textRef.current.value = "";
    }
  }

  return (
    <>
      <div className="input-box">
        <textarea
          className="input-textarea"
          ref={textRef}
          autocorrect="off"
          spellcheck={false}
          placeholder={props.disable ? "Please wait..." : "How can I help you?"}
          disabled={props.disable}
          onKeyUp={(e) => {
            // Submit on Enter but newline on Shift+Enter
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              submit();
            }
          }}
        ></textarea>
        <i onClick={submit} className="send-button codicon codicon-send"></i>
      </div>
      <div className="input-box-divider" ref={hrRef} />
    </>
  );
}

/**
 * A user message in the chat history.
 */
function UserMessage(props: { content: string }) {
  return (
    <div className="right-message-row">
      <div className="user-message">
        <Markdown markdown={props.content} />
      </div>
    </div>
  );
}

/**
 * An assistant message in the chat history.
 */
function AssistantMessage(props: { content: string }) {
  return (
    <div className="left-message-row">
      <div className="assistant-message">
        <Markdown markdown={props.content}></Markdown>
      </div>
    </div>
  );
}

/**
 * A widget in the chat history.
 *
 * A widget "message" is not a real message in the chat: it's
 * inserted into the chat by tool calls, and displayed
 * in the client, but it's not included in the chat history
 * payload that gets sent to the service.
 */
function Widget(props: { histogram: HistogramData }) {
  const histoMap: Map<string, number> = new Map(props.histogram.buckets);
  return (
    <div className="left-message-row">
      <div className="histogram-container">
        <Histogram
          data={histoMap}
          filter=""
          shotCount={props.histogram.shotCount}
          onFilter={() => undefined}
          shotsHeader={false}
        />
      </div>
    </div>
  );
}

type ChatViewModel = {
  /**
   * The complete chat history.
   */
  history: ChatElement[];

  /**
   * The current status, e.g. "ready", "waitingForAssistantResponse", etc.
   */
  status: Status;

  /**
   * Available service backends defined in the configuration.
   */
  serviceOptions: string[];

  /**
   * Service backend in use.
   */
  service?: string;

  /**
   * Any in progress assistant response.
   */
  inProgressResponse: string;
};

/**
 * Copilot command to add a user message to the
 * current chat and request a response.
 */
function submitUserMessage(content: string) {
  postMessageToExtension({
    command: "submitUserMessage",
    request: content,
  });
}

/**
 * Copilot command to restart the chat with a new history.
 * The service backend can be changed here as well.
 */
function restartChat(history: ChatElement[], service?: string) {
  postMessageToExtension({
    command: "restartChat",
    history,
    service,
  });
}

/**
 * Holds the current UI state.
 */
const model: ChatViewModel = {
  history: [],
  inProgressResponse: "",
  status: { status: "ready" },
  service: undefined,
  serviceOptions: [],
};

document.addEventListener("DOMContentLoaded", loaded);
window.addEventListener("message", onMessage);

function loaded() {
  restartChat([], undefined);
}

/**
 * Handles updates to the copilot state.
 */
function onMessage(event: MessageEvent<CopilotUpdate>) {
  const message = event.data;
  switch (message.kind) {
    case "appendDelta":
      model.status = message.payload.status;
      model.inProgressResponse += message.payload.delta;
      break;
    case "updateStatus":
      model.status = message.payload.status;
      break;
    case "updateChat":
      model.inProgressResponse = "";
      model.history = message.payload.history;
      model.status = message.payload.status;
      model.service = message.payload.service;
      model.serviceOptions = message.payload.serviceOptions;
      break;
    default:
      console.error("Unknown message kind: ", (message as any).kind);
      return;
  }

  render(<App model={model} />, document.body);
}

// Wrapper around `postMessage`, just exists to typecheck against `MessageToCopilot`
function postMessageToExtension(message: CopilotCommand) {
  vscodeApi.postMessage(message);
}
