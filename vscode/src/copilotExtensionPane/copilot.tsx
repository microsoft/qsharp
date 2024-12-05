// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

// Use esbuild to bundle and copy the CSS files to the output directory.
import "modern-normalize/modern-normalize.css";
import "highlight.js/styles/default.css";
import "./copilot.css";

import { render } from "preact";

// Set up the Markdown renderer with KaTeX support
import mk from "@vscode/markdown-it-katex";
import markdownIt from "markdown-it";

import hljs from "highlight.js";

import { Histogram, Markdown, ResultsTable } from "qsharp-lang/ux";
import { setRenderer } from "qsharp-lang/ux";
import { useEffect, useRef, useState } from "preact/hooks";

import hlsjQsharp from "./hlsj-qsharp";
import samples, { mock_stream } from "./copilot-samples";

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
      <div style="height: 8px" ref={hrRef} />
    </>
  );
}

function Response(props: { request: string; response: string }) {
  const parts: Array<string | any> = [];

  const histoMap = new Map<string, number>([
    ["000", 5],
    ["001", 1],
    ["010", 20],
    ["011", 18],
    ["100", 1],
    ["101", 0],
    ["110", 3],
    ["111", 1],
  ]);

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
            return (
              <Histogram
                data={histoMap}
                filter=""
                shotCount={100}
                onFilter={() => undefined}
                shotsHeader={false}
              />
            );
          } else if (part.startsWith("```widget\nResults")) {
            return (
              <ResultsTable
                columnNames={["Name", "Date", "Run time", "Cost"]}
                initialColumns={[0, 1, 2, 3]}
                rows={[
                  {
                    color: "blue",
                    cells: ["Carbon-1a", "2024-09-01", "00:01:60", "$25.00"],
                  },
                  {
                    color: "blue",
                    cells: ["Carbon-2b", "2024-09-01", "00:03:04", "$91.50"],
                  },
                  {
                    color: "blue",
                    cells: ["Carbon-3b", "2024-09-02", "00:05:54", "$120.10"],
                  },
                  {
                    color: "blue",
                    cells: ["Test-run", "2024-09-03", "00:00:04", "$00.10"],
                  },
                  {
                    color: "blue",
                    cells: ["qrng", "2024-09-04", "00:00:45", "$1.50"],
                  },
                ]}
                onRowDeleted={() => undefined}
                onRowSelected={() => undefined}
                selectedRow={null}
              />
            );
          }
          {
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
  inProgress: boolean;
};

function App({ state }: { state: CopilotState }) {
  // const [state, setState] = useState<QA[]>([]);

  function onSubmit(text: string) {
    // const newQA: QA = { request: text, response: "" };

    // let gen: Generator<string>;
    // if (text.includes("code")) {
    //   gen = mock_stream(samples.code);
    // } else if (text.includes("noise")) {
    //   gen = mock_stream(samples.noise);
    // } else if (text.includes("python")) {
    //   gen = mock_stream(samples.azure);
    // } else {
    //   gen = mock_stream(samples.jobs);
    // }

    copilotRequest(text);

    // function onChunk() {
    //   const chunk = gen.next();
    //   if (!chunk.done) {
    //     newQA.response += chunk.value;

    //     // Clone into new state
    //     setState([...state, newQA]);
    //     setTimeout(onChunk, 50);
    //   } else {
    //     //(window as any).hljs.highlightAll();
    //   }
    // }
    // onChunk();
  }

  ////////////////////// mineyalc
  function copilotRequest(text: string) {
    // const questionText = document.querySelector(
    //   "#copilotQuestion",
    // ) as HTMLInputElement;
    vscodeApi.postMessage({
      command: "copilotRequest",
      request: text,
    });
    globalState.qas.push({
      request: text,
      response: "",
    });
    globalState.inProgress = true;
    // questionText.value = "";
    render(<App state={state} />, document.body);
  }

  ///////////////// end mineyalc

  return (
    <div style="max-width: 800px; font-size: 0.9em;">
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
      <InputBox onSubmit={onSubmit} inProgress={state.inProgress} />
    </div>
  );
}

let globalState: CopilotState = { tidbits: [], qas: [], inProgress: false };

function loaded() {
  render(<App state={globalState} />, document.body);
}

document.addEventListener("DOMContentLoaded", loaded);
window.addEventListener("message", onMessage);

function onMessage(event: any) {
  const message = event.data;
  // if (!message?.command) {
  //   console.error("Unknown message: ", message);
  //   return;
  // }
  switch (message.command) {
    case "copilotResponseDelta":
      // After a copilot response from the service, but before any tool calls are executed.
      {
        // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
        let cleanedResponse = message.response;
        cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
        cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");
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
        globalState.tidbits = [];
        globalState.qas.push({
          request: message.request ?? "",
          response: cleanedResponse,
        });
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
        // state.qas.push({ request: "", response: "\n\n---\n\n" });
        globalState.inProgress = false;
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
