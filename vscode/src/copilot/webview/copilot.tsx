// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

// Use esbuild to bundle and copy the CSS files to the output directory.
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
            className="welcome-message"
            markdown={`# Azure Quantum<br/>Copilot

I'm here to assist you with your Azure Quantum workspace and help you explore Q# programming.

Try:

_Can you submit this Q# program to Azure Quantum for execution?_

_What are the currently available hardware providers for my workspace?_

_Can you show the results from my last job?_

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
      {model.debugUi.show ? (
        <>
          <ShowPayload
            history={model.history}
            service={model.debugUi.service}
            serviceOptions={model.debugUi.serviceOptions}
            restartChat={restartChat}
          />
          <ServiceSelector
            service={model.debugUi.service}
            serviceOptions={model.debugUi.serviceOptions}
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
        if (model.debugUi.show) {
          elements.push(
            <RetryButton
              history={model.history.slice(0, i + 1)}
              serviceOptions={model.debugUi.serviceOptions}
              selectedService={model.debugUi.service}
              restartChat={restartChat}
              disabled={
                model.status.status !== "ready" &&
                model.status.status !== "assistantConnectionError"
              }
            ></RetryButton>,
          );
        }

        break;
      case "tool":
        if (model.debugUi.show) {
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
        <div className="content-reminder">
          AI generated content may be incorrect.
        </div>
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
   * Any in progress assistant response.
   */
  inProgressResponse: string;

  /**
   * Debug controls to be used in development.
   */
  debugUi:
    | {
        show: true;

        /**
         * Available service backends defined in the configuration.
         */
        serviceOptions: string[];

        /**
         * Service backend in use.
         */
        service?: string;
      }
    | {
        show: false;
      };
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
  debugUi: { show: false },
};

document.addEventListener("DOMContentLoaded", loaded);
window.addEventListener("message", onMessage);

function loaded() {
  setThemeStylesheet();
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
      model.debugUi = message.payload.debugUi;
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

const themeAttribute = "data-vscode-theme-kind";

function updateHljsTheme() {
  let isDark = true;

  const themeType = document.body.getAttribute(themeAttribute);

  switch (themeType) {
    case "vscode-light":
    case "vscode-high-contrast-light":
      isDark = false;
      break;
    default:
      isDark = true;
  }

  // Update the stylesheet href
  document.head.querySelectorAll("link").forEach((el) => {
    const ref = el.getAttribute("href");
    if (ref && ref.includes("hljs")) {
      const newVal = ref.replace(
        /(dark\.css)|(light\.css)/,
        isDark ? "dark.css" : "light.css",
      );
      el.setAttribute("href", newVal);
    }
  });
}

function setThemeStylesheet() {
  // We need to add the right Markdown style-sheet for the theme.

  // For VS Code, there will be an attribute on the body called
  // "data-vscode-theme-kind" that is "vscode-light" or "vscode-high-contrast-light"
  // for light themes, else assume dark (will be "vscode-dark" or "vscode-high-contrast").

  // Use a [MutationObserver](https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver)
  // to detect changes to the theme attribute.
  const callback = (mutations: MutationRecord[]) => {
    for (const mutation of mutations) {
      if (mutation.attributeName === themeAttribute) {
        updateHljsTheme();
      }
    }
  };
  const observer = new MutationObserver(callback);
  observer.observe(document.body, { attributeFilter: [themeAttribute] });

  // Run it once for initial value
  updateHljsTheme();
}
