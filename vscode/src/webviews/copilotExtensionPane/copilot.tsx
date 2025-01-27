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
  QuantumChatMessage,
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
  const responseParts = [];
  const response = props.response;

  console.log(response);

  const beginWidgetMarker = "```widget\n";
  const endWidgetMarker = "\n```\n";

  const beginWidget = response.indexOf(beginWidgetMarker);

  if (beginWidget >= 0) {
    const before = response.slice(0, beginWidget);
    if (before.trim().length > 0) {
      responseParts.push(
        <div className="responseBox">
          <Markdown markdown={before}></Markdown>
        </div>,
      );
    }
    let endWidget = response.indexOf(
      endWidgetMarker,
      beginWidget + beginWidgetMarker.length,
    );

    if (endWidget <= beginWidget + beginWidgetMarker.length) {
      endWidget = response.length;
    }

    const widget = response.slice(
      beginWidget + beginWidgetMarker.length,
      endWidget,
    );

    // the widget
    if (widget.startsWith("Histogram\n")) {
      const histo = JSON.parse(widget.slice("Histogram\n".length));
      if (histo.buckets && typeof histo.shotCount === "number") {
        const histoMap: Map<string, number> = new Map(histo.buckets);
        responseParts.push(
          <div className="histogram-container">
            <Histogram
              data={histoMap}
              filter=""
              shotCount={histo.shotCount}
              onFilter={() => undefined}
              shotsHeader={false}
            />
          </div>,
        );
      }
    }

    const after = response.slice(endWidget + endWidgetMarker.length);
    if (after.trim().length > 0) {
      responseParts.push(
        <div className="responseBox">
          <Markdown markdown={after}></Markdown>
        </div>,
      );
    }
  } else {
    responseParts.push(
      <div className="responseBox">
        <Markdown markdown={response}></Markdown>
      </div>,
    );
  }

  return <div className="response-container">{responseParts}</div>;
}

function RetryButton(props: {
  service: ServiceTypes;
  retryRequest: (service: ServiceTypes) => void;
}) {
  const serviceDropdown = useRef<HTMLSelectElement>(null);
  return (
    // TODO: comment out for demo
    <></>
    // <div style="margin: 10px 0 10px 32px; text-align: right;">
    //   <select ref={serviceDropdown} value={props.service}>
    //     <option value="OpenAI">OpenAI</option>
    //     <option value="AzureQuantumTest">AzureQuantumTest</option>
    //     <option value="AzureQuantumLocal">AzureQuantumLocal</option>
    //   </select>
    //   <button
    //     onClick={() => {
    //       const dropdown = serviceDropdown.current!;
    //       const selectedService = dropdown.value;
    //       props.retryRequest(selectedService as ServiceTypes);
    //     }}
    //   >
    //     Try Again
    //   </button>
    // </div>
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
  history: QuantumChatMessage[];
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
    globalState.history.push({
      role: "user",
      content: text,
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
          {/* <a
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
          </a> */}
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
      {/* <div style="height: 30px;">
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
      </div> */}
    </div>
  );
}

let globalState: CopilotState = {
  tidbits: [],
  inProgress: false,
  toolInProgress: null,
  // service: "AzureQuantumLocal", // default
  service: "OpenAI", // TODO: default for demo
  conversation: [],
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
      elements.push(<ResponseBox response={message.response} />);
    } else {
      elements.push(
        <div className="request-container">
          <ConversationMessage message={message} />
        </div>,
      );
    }
  }

  if (state.tidbits.length > 0) {
    elements.push(<ResponseBox response={state.tidbits.join("")} />);
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
      globalState.toolInProgress = null;
      globalState.tidbits = [];
      globalState.conversation.push({
        role: "assistant",
        response: message.payload.response,
      });
      globalState.history = message.payload.history;
      break;
    case "copilotToolCall":
      {
        globalState.toolInProgress = message.payload.toolName;
      }
      break;
    case "copilotToolCallDone":
      {
        // TODO: comment out for demo since this doesn't look good yet

        // const toolName = message.payload.toolName;
        // const args = JSON.stringify(message.payload.args, undefined, 2);
        // const result = JSON.stringify(message.payload.result, undefined, 2);
        // globalState.conversation.push({
        //   role: "tool",
        //   name: toolName,
        //   args: args,
        //   result: result,
        // });
        globalState.history = message.payload.history;
        globalState.toolInProgress = null;
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
