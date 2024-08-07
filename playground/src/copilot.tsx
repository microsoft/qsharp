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

import { Markdown } from "qsharp-lang/ux";
import { setRenderer } from "qsharp-lang/ux";
import { useRef, useState } from "preact/hooks";

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
md.use((mk as any).default, {
  enableMathBlockInHtml: true,
  enableMathInlineInHtml: true,
});
setRenderer((input: string) => md.render(input));

function InputBox(props: { onSubmit: (text: string) => void }) {
  const textRef = useRef<HTMLTextAreaElement>(null);

  function submit() {
    if (textRef.current) {
      props.onSubmit(textRef.current.value);
      textRef.current.value = "";
    }
  }

  return (
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
  );
}

function Response(props: { request: string; response: string }) {
  return (
    <div>
      <div class="requestBox">
        <Markdown markdown={props.request} />
      </div>
      <div>
        <Markdown markdown={props.response}></Markdown>
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

    const gen = mock_stream(samples.general);

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
    <div style="max-width: 800px">
      <h1>Welcome to Quantum Copilot</h1>
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
