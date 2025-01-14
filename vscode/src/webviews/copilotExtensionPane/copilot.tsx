// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

// Use esbuild to bundle and copy the CSS files to the output directory.
import "modern-normalize/modern-normalize.css";
import "highlight.js/styles/default.css";
import "./copilot.css";
import { MessageToCopilot } from "../../commonTypes";

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
    if (textRef.current) {
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
          onKeyUp={(e) => e.key === "Enter" && submit()}
        ></textarea>
        <svg
          onClick={submit}
          focusable="false"
          viewBox="0 0 16 16"
          width="16"
          height="16"
        >
          <path d="M.989 8 .064 2.68a1.342 1.342 0 0 1 1.85-1.462l13.402 5.744a1.13 1.13 0 0 1 0 2.076L1.913 14.782a1.343 1.343 0 0 1-1.85-1.463L.99 8Zm.603-5.288L2.38 7.25h4.87a.75.75 0 0 1 0 1.5H2.38l-.788 4.538L13.929 8Z"></path>
        </svg>
      </div>
      <div style="height: 50px" ref={hrRef} />
    </>
  );
}

function Response(props: { request: string; response: string }) {
  const parts: Array<string | any> = [];

  const widget = props.response.indexOf("```widget\n");
  if (widget >= 0) {
    parts.push(props.response.slice(0, widget));
    let endWidget = props.response.indexOf("\n```\n", widget + 9);
    if (endWidget < 0 || endWidget >= props.response.length - 4) {
      endWidget = props.response.length;
      parts.push(props.response.slice(widget));
    } else {
      parts.push(props.response.slice(widget, endWidget + 4));
      parts.push(props.response.slice(endWidget + 4));
    }
  } else {
    parts.push(props.response);
  }

  const toolCallPrefix = "Executing: ";

  return (
    <div>
      {props.request ? (
        <div class="requestBox">
          <Markdown markdown={props.request} />
        </div>
      ) : null}
      <div class="responseBox">
        {parts.map((part) => {
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
          } else if (props.response.startsWith(toolCallPrefix)) {
            const tool = props.response.slice(toolCallPrefix.length);
            return (
              <div style="font-weight: bold; text-align: right; font-size: smaller;">
                {tool}
              </div>
            );
          } else {
            return <Markdown markdown={part}></Markdown>;
          }
        })}
      </div>
    </div>
  );
}

type QA = {
  request: string;
  response: string;
};

type CopilotState = {
  tidbits: string[];
  qas: QA[];
  toolInProgress: string | null;
  inProgress: boolean;
  service: "AzureQuantumLocal" | "AzureQuantumTest" | "OpenAI";
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
      qas: [],
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
    globalState.qas.push({
      request: text,
      response: "",
    });
    globalState.inProgress = true;
    render(<App state={state} />, document.body);
  }

  const historyRef = useRef<HTMLPreElement>(null);

  return (
    <div style="max-width: 800px; font-size: 0.9em; display: flex; flex-direction: column; height: 100%;">
      <div style="flex: 1; ">
        <h2 style="margin-top: 0">Welcome to Quantum Copilot</h2>
        {(state as CopilotState).qas.map((qa) => (
          <Response request={qa.request} response={qa.response} />
        ))}
        {
          <Response
            request={""}
            response={(state as CopilotState).tidbits.join("")}
          />
        }
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
          <pre ref={historyRef} style="display: none;">
            {JSON.stringify(state.history, undefined, 2)}
          </pre>
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
  qas: [],
  inProgress: false,
  toolInProgress: null,
  service: "AzureQuantumLocal", // default
  history: [],
};

function loaded() {
  render(<App state={globalState} />, document.body);
}

document.addEventListener("DOMContentLoaded", loaded);
window.addEventListener("message", onMessage);

function onMessage(event: any) {
  const message = event.data;
  switch (message.command) {
    case "copilotResponseDelta":
      // After a copilot response from the service, but before any tool calls are executed.
      {
        // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
        let cleanedResponse = message.response;
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
        let cleanedResponse = message.response;
        cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
        cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");
        globalState.toolInProgress = null;
        globalState.tidbits = [];
        globalState.qas.push({
          request: message.request ?? "",
          response: cleanedResponse,
        });
      }
      break;
    case "copilotToolCall":
      {
        globalState.toolInProgress = message.toolName;
        // globalState.qas.push({
        //   request: message.request ?? "",
        //   response: "Executing: " + message.toolName,
        // });
      }
      break;
    case "copilotResponseHistogram":
      {
        if (!message.buckets || typeof message.shotCount !== "number") {
          console.error("No buckets in message: ", message);
          return;
        }
        const buckets = message.buckets as Array<[string, number]>;
        const histogram = JSON.stringify({
          buckets: buckets,
          shotCount: message.shotCount,
        });
        globalState.qas.push({
          request: "",
          response: "```widget\nHistogram\n" + histogram,
        });
      }
      break;
    case "copilotResponseDone":
      // After all the events in a single response stream have been received
      {
        globalState.inProgress = false;
        globalState.toolInProgress = null;
        globalState.history = message.history;
      }
      // Highlight any code blocks
      // Need to wait until Markdown is rendered. Hack for now with a timeout
      setTimeout(() => {
        (window as any).hljs.highlightAll();
      }, 100);
      break;
    default:
      console.error("Unknown command: ", message.command);
      return;
  }
  // vscodeApi.setState(state);
  render(<App state={globalState} />, document.body);
}

function sendMessageToExtension(message: MessageToCopilot) {
  vscodeApi.postMessage(message);
}
