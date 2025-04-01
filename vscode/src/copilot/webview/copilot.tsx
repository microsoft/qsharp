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
import hljs from "highlight.js/lib/core";
import CopyButtonPlugin from "highlightjs-copy";
import "highlightjs-copy/styles/highlightjs-copy.css";
import python from "highlight.js/lib/languages/python";
import bash from "highlight.js/lib/languages/bash"; // Used sometimes for plain text
import markdownIt from "markdown-it";
import { useEffect, useRef } from "preact/hooks";
import { Histogram, Markdown, setRenderer } from "qsharp-lang/ux";
import {
  RetryButton,
  ServiceSelector,
  ShowPayload,
  ToolMessage,
} from "./debugUi";
import hlsjQsharp from "./hljs-qsharp";
import { WebviewApi } from "vscode-webview";

const vscodeApi: WebviewApi<ChatElement[]> = acquireVsCodeApi();

// Only include a small set of languages so as not
// to bloat the code
hljs.registerLanguage("python", python);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("qsharp", hlsjQsharp);
hljs.addPlugin(new CopyButtonPlugin());
const md = markdownIt("commonmark");
md.use(mk as any, {
  enableMathBlockInHtml: true,
  enableMathInlineInHtml: true,
});
setRenderer((input: string) => md.render(input));

function Welcome() {
  return (
    <div id="qs-copilot-welcome">
      <div class="qs-copilot-title">
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M17.0722 3.66246C16.7827 2.67691 15.8784 2 14.8512 2L14.1735 2C13.0569 2 12.0994 2.7971 11.897 3.8952L10.7119 10.3247L11.0335 9.22215C11.3216 8.23453 12.2269 7.55555 13.2557 7.55555L17.1772 7.55556L18.8242 8.19709L20.4119 7.55556H19.9483C18.9212 7.55556 18.0168 6.87864 17.7273 5.89309L17.0722 3.66246Z"
            fill="url(#paint0_radial_301_554)"
          ></path>
          <path
            d="M7.16561 20.328C7.45189 21.3183 8.35852 22 9.38937 22H10.8432C12.0912 22 13.1145 21.0107 13.1567 19.7634L13.3712 13.4201L12.9681 14.7851C12.6776 15.7691 11.774 16.4444 10.7481 16.4444L6.78679 16.4444L5.37506 15.6786L3.84668 16.4444H4.3025C5.33335 16.4444 6.23998 17.1261 6.52626 18.1164L7.16561 20.328Z"
            fill="url(#paint1_radial_301_554)"
          ></path>
          <path
            d="M14.7507 2H6.73041C4.43891 2 3.06401 5.02777 2.14741 8.05553C1.06148 11.6426 -0.359484 16.4401 3.75146 16.4401H7.21482C8.24955 16.4401 9.15794 15.7559 9.44239 14.7611C10.0445 12.6551 11.0997 8.98146 11.9285 6.18489C12.3497 4.76367 12.7005 3.5431 13.239 2.783C13.5409 2.35686 14.044 2 14.7507 2Z"
            fill="url(#paint2_linear_301_554)"
          ></path>
          <path
            d="M14.7507 2H6.73041C4.43891 2 3.06401 5.02777 2.14741 8.05553C1.06148 11.6426 -0.359484 16.4401 3.75146 16.4401H7.21482C8.24955 16.4401 9.15794 15.7559 9.44239 14.7611C10.0445 12.6551 11.0997 8.98146 11.9285 6.18489C12.3497 4.76367 12.7005 3.5431 13.239 2.783C13.5409 2.35686 14.044 2 14.7507 2Z"
            fill="url(#paint3_linear_301_554)"
          ></path>
          <path
            d="M9.24951 22H17.2698C19.5613 22 20.9362 18.9722 21.8528 15.9445C22.9387 12.3574 24.3597 7.55991 20.2487 7.55991H16.7854C15.7506 7.55991 14.8422 8.24407 14.5578 9.23894C13.9556 11.3449 12.9005 15.0186 12.0717 17.8151C11.6505 19.2363 11.2996 20.4569 10.7612 21.217C10.4593 21.6431 9.95619 22 9.24951 22Z"
            fill="url(#paint4_radial_301_554)"
          ></path>
          <path
            d="M9.24951 22H17.2698C19.5613 22 20.9362 18.9722 21.8528 15.9445C22.9387 12.3574 24.3597 7.55991 20.2487 7.55991H16.7854C15.7506 7.55991 14.8422 8.24407 14.5578 9.23894C13.9556 11.3449 12.9005 15.0186 12.0717 17.8151C11.6505 19.2363 11.2996 20.4569 10.7612 21.217C10.4593 21.6431 9.95619 22 9.24951 22Z"
            fill="url(#paint5_linear_301_554)"
          ></path>
          <defs>
            <radialGradient
              id="paint0_radial_301_554"
              cx="0"
              cy="0"
              r="1"
              gradientUnits="userSpaceOnUse"
              gradientTransform="translate(18.9994 10.3791) rotate(-128.978) scale(8.73886 8.198)"
            >
              <stop offset="0.0955758" stop-color="#0078D4"></stop>
              <stop offset="0.715277" stop-color="#0C709B"></stop>
              <stop offset="1" stop-color="#0A5079"></stop>
            </radialGradient>
            <radialGradient
              id="paint1_radial_301_554"
              cx="0"
              cy="0"
              r="1"
              gradientUnits="userSpaceOnUse"
              gradientTransform="translate(5.57463 16.2453) rotate(45.7) scale(8.04078 7.90145)"
            >
              <stop stop-color="#0091EB"></stop>
              <stop offset="0.523516" stop-color="#2764E7"></stop>
              <stop offset="0.923392" stop-color="#0636C3"></stop>
            </radialGradient>
            <linearGradient
              id="paint2_linear_301_554"
              x1="5.16831"
              y1="2"
              x2="7.75605"
              y2="17.2359"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0.289817" stop-color="#00A5D9"></stop>
              <stop offset="0.662336" stop-color="#21CAB2"></stop>
              <stop offset="0.950002" stop-color="#6ADC90"></stop>
            </linearGradient>
            <linearGradient
              id="paint3_linear_301_554"
              x1="7.25046"
              y1="2"
              x2="7.87502"
              y2="16.4401"
              gradientUnits="userSpaceOnUse"
            >
              <stop stop-color="#10C9EC"></stop>
              <stop
                offset="0.166667"
                stop-color="#01AEE4"
                stop-opacity="0"
              ></stop>
            </linearGradient>
            <radialGradient
              id="paint4_radial_301_554"
              cx="0"
              cy="0"
              r="1"
              gradientUnits="userSpaceOnUse"
              gradientTransform="translate(20.6607 6.14612) rotate(111.466) scale(19.1552 22.9833)"
            >
              <stop offset="0.154405" stop-color="#2771D8"></stop>
              <stop offset="0.678875" stop-color="#14B1FF"></stop>
              <stop offset="0.931138" stop-color="#16BFDF"></stop>
            </radialGradient>
            <linearGradient
              id="paint5_linear_301_554"
              x1="21.2403"
              y1="7.00944"
              x2="20.306"
              y2="12.5796"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0.0581535" stop-color="#14B1FF"></stop>
              <stop
                offset="0.708063"
                stop-color="#2976DB"
                stop-opacity="0"
              ></stop>
            </linearGradient>
          </defs>
        </svg>
        <div style="margin-left: 8px;">Microsoft Quantum Copilot</div>
      </div>
      <div class="qs-copilot-blurb">
        Welcome! Microsoft Quantum Copilot is designed to help you develop and
        run quantum programs. You can ask questions such as:
      </div>
      <div class="qs-copilot-demo1">
        Can you help me implement Grover's search?
      </div>
      <div class="qs-copilot-demo1" style="margin-left: 1.5em">
        Please submit this job to my Azure Quantum workspace
      </div>
      <div class="qs-copilot-demo1">
        Show me the results for the last job as a histogram
      </div>
      <div class="qs-copilot-disclaimer">
        Copilot is powered by AI, so mistakes are possible. Review output
        carefully before use.
      </div>
    </div>
  );
}

function App({ model }: { model: ChatViewModel }) {
  return (
    <div className="app">
      <div className="chat">
        {model.history.length > 0 ? <ChatHistory model={model} /> : <Welcome />}
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
      {status.status === "waitingAssistantResponse" ? (
        <span class="codicon codicon-loading codicon-modifier-spin"></span>
      ) : status.status === "executingTool" ? (
        <>
          <span>{status.toolName}&nbsp;</span>
          <span class="codicon codicon-loading codicon-modifier-spin"></span>
        </>
      ) : status.status === "assistantConnectionError" ? (
        "There was an error communicating with Microsoft Quantum Copilot. Please check your Internet connection and try again."
      ) : status.status === "awaitingConfirmation" ? (
        <>
          <div>{status.confirmText}</div>
          <div>
            <button
              type="button"
              class="confirm-button"
              onClick={() => submitConfirmation(true)}
            >
              Yes
            </button>
            <button
              type="button"
              class="confirm-button"
              onClick={() => submitConfirmation(false)}
            >
              No
            </button>
          </div>
        </>
      ) : (
        ""
      )}
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
  // Highlight.js needs to be called on the code blocks after they are present in the
  // DOM for the highlighjs-copy plugin to work (as it needs to add buttons to the container).
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    ref.current?.querySelectorAll("pre code").forEach((block) => {
      hljs.highlightElement(block as HTMLElement);
    });
  }, [props.content]);

  return (
    <div className="left-message-row">
      <div className="assistant-message">
        <div ref={ref}>
          <Markdown markdown={props.content}></Markdown>
        </div>
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

function submitConfirmation(confirmed: boolean) {
  postMessageToExtension({ command: "confirmation", confirmed });
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
  const savedHistory = vscodeApi.getState();
  restartChat(savedHistory ?? [], undefined);
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
      vscodeApi.setState(message.payload.history);
      break;
    case "showConfirmation":
      model.status = {
        status: "awaitingConfirmation",
        confirmText: message.payload.confirmText,
      };
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
