// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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

function InputBox(props: { onSubmit: (text: string) => void }) {
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
          placeholder="How can I help you?"
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
      <div class="requestBox">
        <Markdown markdown={props.request} />
      </div>
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

function App() {
  const [state, setState] = useState<QA[]>([]);

  function onSubmit(text: string) {
    const newQA: QA = { request: text, response: "" };

    let gen: Generator<string>;
    if (text.includes("code")) {
      gen = mock_stream(samples.code);
    } else if (text.includes("noise")) {
      gen = mock_stream(samples.noise);
    } else if (text.includes("python")) {
      gen = mock_stream(samples.azure);
    } else {
      gen = mock_stream(samples.jobs);
    }

    function onChunk() {
      const chunk = gen.next();
      if (!chunk.done) {
        newQA.response += chunk.value;

        // Clone into new state
        setState([...state, newQA]);
        setTimeout(onChunk, 50);
      } else {
        //(window as any).hljs.highlightAll();
      }
    }
    onChunk();
  }

  return (
    <div style="max-width: 800px; font-size: 0.9em;">
      <h2 style="margin-top: 0">Welcome to Quantum Copilot</h2>
      {state.map((qa) => (
        <Response request={qa.request} response={qa.response} />
      ))}
      <InputBox onSubmit={onSubmit} />
    </div>
  );
}

function loaded() {
  render(<App />, document.body);
}

document.addEventListener("DOMContentLoaded", loaded);
