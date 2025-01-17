// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

// Use esbuild to bundle and copy the CSS files to the output directory.
import "modern-normalize/modern-normalize.css";
import "highlight.js/styles/default.css";
import "./copilot.css";
import {
  CopilotEvent,
  MessageToCopilot,
  ServiceTypes,
} from "../../commonTypes";

import { render } from "preact";

// Set up the Markdown renderer with KaTeX support
import mk from "@vscode/markdown-it-katex";
import markdownIt from "markdown-it";

import hljs from "highlight.js";

import { Histogram, Markdown } from "qsharp-lang/ux";
import { setRenderer } from "qsharp-lang/ux";
import { useEffect, useRef } from "preact/hooks";

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

function InputBox(props: {
  onSubmit: (text: string) => void;
  inProgress: boolean;
}) {
  const textRef = useRef<HTMLTextAreaElement>(null);
  const hrRef = useRef<HTMLHRElement>(null);

  useEffect(() => {
    hrRef.current?.scrollIntoView(false);
    // TODO: bring this back, it's annoying

    // textRef.current?.focus();
  });

  function submit() {
    if (
      textRef.current &&
      textRef.current.value?.trim().length > 0 &&
      !props.inProgress
    ) {
      props.onSubmit(textRef.current.value);
      textRef.current.value = "";
    }
  }

  return (
    <>
      <div class="inputDiv">
        <textarea
          ref={textRef}
          autocorrect="off"
          spellcheck={false}
          placeholder={
            props.inProgress ? "Please wait..." : "How can I help you?"
          }
          disabled={props.inProgress}
          onKeyUp={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              submit();
            }
          }}
        ></textarea>
        <i
          onClick={submit}
          className="codicon codicon-send"
          style="padding: 2px;"
        ></i>
        {/* <svg
          onClick={submit}
          focusable="false"
          viewBox="0 0 16 16"
          width="16"
          height="16"
        >
          <path d="M.989 8 .064 2.68a1.342 1.342 0 0 1 1.85-1.462l13.402 5.744a1.13 1.13 0 0 1 0 2.076L1.913 14.782a1.343 1.343 0 0 1-1.85-1.463L.99 8Zm.603-5.288L2.38 7.25h4.87a.75.75 0 0 1 0 1.5H2.38l-.788 4.538L13.929 8Z"></path>
        </svg> */}
      </div>
      <div style="height: 50px" ref={hrRef} />
    </>
  );
}

function ResponseBox(props: { response: string }) {
  const responseParts: string[] = [];
  const response = props.response;

  const widget = response.indexOf("```widget\n");
  if (widget >= 0) {
    responseParts.push(response.slice(0, widget));
    let endWidget = response.indexOf("\n```\n", widget + 9);
    if (endWidget < 0 || endWidget >= response.length - 4) {
      endWidget = response.length;
      responseParts.push(response.slice(widget));
    } else {
      responseParts.push(response.slice(widget, endWidget + 4));
      responseParts.push(response.slice(endWidget + 4));
    }
  } else {
    responseParts.push(response);
  }

  return (
    <div class="responseBox">
      {responseParts.map((part) => {
        if (part.startsWith("```widget\nHistogram")) {
          const histo = JSON.parse(part.slice(20));
          if (histo.buckets && typeof histo.shotCount === "number") {
            const histoMap: Map<string, number> = new Map(histo.buckets);
            return (
              <Histogram
                data={histoMap}
                filter=""
                shotCount={histo.shotCount}
                onFilter={() => undefined}
                shotsHeader={false}
              />
            );
          }
        } else {
          return <Markdown markdown={part}></Markdown>;
        }
      })}
    </div>
  );
}

function RetryButton(props: {
  service: ServiceTypes;
  retryRequest: (service: ServiceTypes) => void;
}) {
  const serviceDropdown = useRef<HTMLSelectElement>(null);
  return (
    <div style="margin: 10px 0 10px 32px; text-align: right;">
      <select ref={serviceDropdown} value={props.service}>
        <option value="OpenAI">OpenAI</option>
        <option value="AzureQuantumTest">AzureQuantumTest</option>
        <option value="AzureQuantumLocal">AzureQuantumLocal</option>
      </select>
      <button
        onClick={() => {
          const dropdown = serviceDropdown.current!;
          const selectedService = dropdown.value;
          props.retryRequest(selectedService as ServiceTypes);
        }}
      >
        Try Again
      </button>
    </div>
  );
}

function ConversationMessage(props: { message: ConversationMessage }) {
  const message = props.message;

  switch (message.role) {
    case "user": {
      return (
        <div class="requestBox">
          <Markdown markdown={message.request} />
        </div>
      );
    }
    case "tool": {
      return (
        // match the widget style from https://github.com/microsoft/vscode/blob/c799d209cd4846a2a822b55dbf2ca21893008faa/src/vs/workbench/contrib/chat/browser/media/chatCodeBlockPill.css#L6
        <div
          style="
        	border: 1px solid var(--vscode-chat-requestBorder, var(--vscode-input-background, transparent));
          border-radius: 4px;
          width: fit-content;
          font-weight: normal;
          text-decoration: none;
          font-size: 11px;
          padding: 0 3px;
          white-space: pre;
        "
        >
          {message.name}({message.args}) =&gt; {message.result}
        </div>
      );
    }
  }
  return <></>;
}

type ConversationMessage =
  | {
      role: "assistant";
      response: string;
    }
  | {
      role: "user";
      request: string;
    }
  | {
      role: "tool";
      name: string;
      args: string;
      result: string;
    };

type CopilotState = {
  tidbits: string[];
  conversation: ConversationMessage[];
  toolInProgress: string | null;
  inProgress: boolean;
  service: ServiceTypes;
  history: object[];
};

function App({ state }: { state: CopilotState }) {
  function reset(ev: any) {
    const service = ev.target.value;

    sendMessageToExtension({
      command: "resetCopilot",
      request: service,
    });
    globalState = {
      tidbits: [],
      conversation: [],
      inProgress: false,
      service,
      toolInProgress: null,
      history: [],
    };
    render(<App state={globalState} />, document.body);
  }

  function onSubmit(text: string) {
    copilotRequest(text);
  }

  function copilotRequest(text: string) {
    vscodeApi.postMessage({
      command: "copilotRequest",
      request: text,
    });
    globalState.conversation.push({
      role: "user",
      request: text,
    });
    globalState.inProgress = true;
    render(<App state={state} />, document.body);
  }

  function retryRequest(service: ServiceTypes) {
    vscodeApi.postMessage({
      command: "retryRequest",
      service,
    });
    globalState.service = service;
    // pop until the last user message - don't pop the user message
    while (globalState.conversation.length > 0) {
      const lastMessage = globalState.conversation.pop();
      const lastHistoryMessage = globalState.history.pop();
      if (lastMessage?.role === "user" && lastHistoryMessage) {
        globalState.conversation.push(lastMessage);
        globalState.history.push(lastHistoryMessage);
        break;
      }
    }
    globalState.inProgress = true;
    render(<App state={state} />, document.body);
  }

  const historyRef = useRef<HTMLDivElement>(null);

  return (
    <div style="max-width: 800px; font-size: 0.9em; display: flex; flex-direction: column; height: 100%;">
      <div style="flex: 1;">
        {FinishedConversation(state, retryRequest)}
        <div
          id="toolStatus"
          style="height: 30px; font-weight: bold; text-align: right; font-size: smaller;"
        >
          {state.inProgress ? "🕒" : ""}
          {state.toolInProgress ? state.toolInProgress : ""}
        </div>
        <InputBox onSubmit={onSubmit} inProgress={state.inProgress} />
        <div>
          <a
            href="#"
            onClick={(e) => {
              e.preventDefault();
              if (historyRef.current) {
                historyRef.current.style.display =
                  historyRef.current.style.display === "none"
                    ? "block"
                    : "none";
              }
            }}
          >
            history
          </a>
          <div ref={historyRef} style="display: none;">
            <a
              href="#"
              onClick={(e) => {
                e.preventDefault();
                navigator.clipboard.writeText(
                  JSON.stringify(state.history, undefined, 2),
                );
                if (e.target) {
                  const a = e.target as HTMLAnchorElement;
                  a.textContent = "copy to clipboard - copied.";
                  setTimeout(() => {
                    a.textContent = "copy to clipboard";
                  }, 2000);
                }
              }}
            >
              copy to clipboard
            </a>
            <Markdown
              markdown={
                "```json\n" +
                JSON.stringify(state.history, undefined, 2) +
                "\n```"
              }
            ></Markdown>
          </div>
        </div>
      </div>
      <div style="height: 30px;">
        <div class="toggle-container">
          <div class="radio-group">
            <label>
              <input
                type="radio"
                name="service"
                value="AzureQuantumTest"
                checked={state.service === "AzureQuantumTest"}
                onChange={reset}
              />
              AzureQuantumTest
            </label>
            <label>
              <input
                type="radio"
                name="service"
                value="AzureQuantumLocal"
                checked={state.service === "AzureQuantumLocal"}
                onChange={reset}
              />
              AzureQuantumLocal
            </label>
            <label>
              <input
                type="radio"
                name="service"
                value="OpenAI"
                checked={state.service === "OpenAI"}
                onChange={reset}
              />
              OpenAI
            </label>
          </div>
        </div>
      </div>
    </div>
  );
}

let globalState: CopilotState = {
  tidbits: [],
  conversation: [],
  inProgress: false,
  toolInProgress: null,
  service: "AzureQuantumLocal", // default
  history: [],
};

function FinishedConversation(
  state: CopilotState,
  retryRequest: (service: ServiceTypes) => void,
) {
  const elements = [];
  // oh my god
  const lastUserMessage =
    state.conversation
      .map((c, index) => (c.role === "user" ? index : -1))
      .filter((index) => index !== -1)
      .pop() ?? -1;

  for (const message of state.conversation) {
    if (message.role === "assistant") {
      elements.push(
        <div className="response-container">
          <ResponseBox response={message.response} />
        </div>,
      );
    } else {
      elements.push(
        <div className="request-container">
          <ConversationMessage message={message} />
        </div>,
      );
    }
  }

  if (state.tidbits.length > 0) {
    elements.push(
      <div className="response-container">
        <ResponseBox response={state.tidbits.join("")} />
      </div>,
    );
  }

  // insert at lastUserMessage
  if (lastUserMessage >= 0) {
    elements.splice(
      lastUserMessage + 1,
      0,
      <RetryButton
        service={state.service}
        retryRequest={retryRequest}
      ></RetryButton>,
    );
  }

  return elements;
}

function loaded() {
  render(<App state={globalState} />, document.body);
}

document.addEventListener("DOMContentLoaded", loaded);
window.addEventListener("message", onMessage);

function onMessage(event: MessageEvent<CopilotEvent>) {
  const message = event.data;
  switch (message.kind) {
    case "copilotResponseDelta":
      // After a copilot response from the service, but before any tool calls are executed.
      {
        // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
        let cleanedResponse = message.payload.response;
        cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
        cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");
        globalState.toolInProgress = null;
        globalState.tidbits.push(cleanedResponse);
      }
      break;
    case "copilotResponse":
      // After a copilot response from the service, but before any tool calls are executed.
      {
        // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
        let cleanedResponse = message.payload.response;
        cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
        cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");
        globalState.toolInProgress = null;
        globalState.tidbits = [];
        globalState.conversation.push({
          role: "assistant",
          response: cleanedResponse,
        });
      }
      break;
    case "copilotToolCall":
      {
        globalState.toolInProgress = message.payload.toolName;
      }
      break;
    case "copilotToolCallDone":
      {
        const toolName = message.payload.toolName;
        const args = JSON.stringify(message.payload.args, undefined, 2);
        const result = JSON.stringify(message.payload.result, undefined, 2);
        globalState.conversation.push({
          role: "tool",
          name: toolName,
          args: args,
          result: result,
        });
        globalState.history = message.payload.history;
        globalState.toolInProgress = null;
      }
      break;
    case "copilotResponseHistogram":
      {
        if (
          !message.payload.buckets ||
          typeof message.payload.shotCount !== "number"
        ) {
          console.error("No buckets in message: ", message);
          return;
        }
        const buckets = message.payload.buckets as Array<[string, number]>;
        const histogram = JSON.stringify({
          buckets: buckets,
          shotCount: message.payload.shotCount,
        });
        globalState.conversation.push({
          role: "assistant",
          response: "```widget\nHistogram\n" + histogram,
        });
      }
      break;
    case "copilotResponseDone":
      // After all the events in a single response stream have been received
      {
        globalState.inProgress = false;
        globalState.toolInProgress = null;
        globalState.history = message.payload.history;
      }
      // Highlight any code blocks
      // Need to wait until Markdown is rendered. Hack for now with a timeout
      setTimeout(() => {
        (window as any).hljs.highlightAll();
      }, 100);
      break;
    default:
      console.error("Unknown message kind: ", (message as any).kind);
      return;
  }
  // vscodeApi.setState(state);
  render(<App state={globalState} />, document.body);
}

function sendMessageToExtension(message: MessageToCopilot) {
  vscodeApi.postMessage(message);
}
