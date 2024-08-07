// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Use esbuild to bundle and copy the CSS files to the output directory.
import "modern-normalize/modern-normalize.css";

import { render } from "preact";

// Set up the Markdown renderer with KaTeX support
import mk from "@vscode/markdown-it-katex";
import markdownIt from "markdown-it";

import { Markdown } from "qsharp-lang/ux";
import { setRenderer } from "qsharp-lang/ux";

const md = markdownIt("commonmark");
md.use((mk as any).default, {
  enableMathBlockInHtml: true,
  enableMathInlineInHtml: true,
});
setRenderer((input: string) => md.render(input));

const sample = String.raw`This is a **Markdown** cell with $\frac{\pi}{4}$ LaTeX support`;

function InputBox() {
  return (
    <div class="inputDiv">
      <textarea
        autocorrect="off"
        spellcheck={false}
        placeholder="How can I help you?"
      ></textarea>
      <svg
        onClick={() => {
          alert("Copilot is not available in the playground.");
        }}
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

function App() {
  return (
    <>
      <h1>Welcome to Quantum Copilot</h1>
      <Markdown markdown={sample}></Markdown>
      <InputBox />
    </>
  );
}

function loaded() {
  render(<App />, document.body);
}

document.addEventListener("DOMContentLoaded", loaded);
