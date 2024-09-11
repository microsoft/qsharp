// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import {
  CircuitPanel,
  CircuitProps,
  EstimatesPanel,
  Histogram,
  Markdown,
  setRenderer,
  type ReData,
} from "qsharp-lang/ux";
import { HelpPage } from "./help";
import { DocumentationView } from "./docview";
import hljsQsharp from "./qsharp-hljs";

import "./webview.css";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - there are no types for this
import mk from "@vscode/markdown-it-katex";
import markdownIt from "markdown-it";
import { useEffect, useRef } from "preact/hooks";
const md = markdownIt("commonmark");
md.use(mk, {
  enableMathBlockInHtml: true,
  enableMathInlineInHtml: true,
});
setRenderer((input: string) => md.render(input));

window.addEventListener("message", onMessage);
window.addEventListener("load", main);

type HistogramState = {
  viewType: "histogram";
  buckets: Array<[string, number]>;
  shotCount: number;
};

type EstimatesState = {
  viewType: "estimates";
  estimatesData: {
    calculating: boolean;
    estimates: ReData[];
  };
};

type CircuitState = {
  viewType: "circuit";
  props: CircuitProps;
};

type DocumentationState = {
  viewType: "documentation";
  fragmentsToRender: string[];
};

type CopilotState = {
  viewType: "copilot";
  qas: QA[];
  inProgress: boolean;
};

type State =
  | { viewType: "loading" }
  | { viewType: "help" }
  | HistogramState
  | EstimatesState
  | CircuitState
  | DocumentationState
  | CopilotState;

const loadingState: State = { viewType: "loading" };
const helpState: State = { viewType: "help" };
let state: State = loadingState;

const themeAttribute = "data-vscode-theme-kind";

function updateGitHubTheme() {
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
    if (ref && ref.includes("github-markdown")) {
      const newVal = ref.replace(
        /(dark\.css)|(light\.css)/,
        isDark ? "dark.css" : "light.css",
      );
      el.setAttribute("href", newVal);
    }

    if (!isDark && el.id === "hljs-dark-theme") {
      // Remove the dark theme
      el.remove();
    }
  });

  if (isDark) {
    // Add a stylesheet with inline css
    const style = document.createElement("style");
    style.id = "hljs-dark-theme";
    style.innerHTML = `pre code.hljs{display:block;overflow-x:auto;padding:1em}code.hljs{padding:3px 5px}.hljs{color:#ddd;background:#303030}.hljs-keyword,.hljs-link,.hljs-literal,.hljs-section,.hljs-selector-tag{color:#fff}.hljs-addition,.hljs-attribute,.hljs-built_in,.hljs-bullet,.hljs-name,.hljs-string,.hljs-symbol,.hljs-template-tag,.hljs-template-variable,.hljs-title,.hljs-type,.hljs-variable{color:#d88}.hljs-comment,.hljs-deletion,.hljs-meta,.hljs-quote{color:#979797}.hljs-doctag,.hljs-keyword,.hljs-literal,.hljs-name,.hljs-section,.hljs-selector-tag,.hljs-strong,.hljs-title,.hljs-type{font-weight:700}.hljs-emphasis{font-style:italic}`;
    document.head.appendChild(style);
  }
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
        updateGitHubTheme();
      }
    }
  };
  const observer = new MutationObserver(callback);
  observer.observe(document.body, { attributeFilter: [themeAttribute] });

  // Run it once for initial value
  updateGitHubTheme();
}

function main() {
  state = (vscodeApi.getState() as any) || loadingState;
  render(<App state={state} />, document.body);
  setThemeStylesheet();
  (window as any).hljs.registerLanguage("qsharp", hljsQsharp);
  vscodeApi.postMessage({ command: "ready" });
}

function onMessage(event: any) {
  const message = event.data;
  if (!message?.command) {
    console.error("Unknown message: ", message);
    return;
  }
  switch (message.command) {
    case "histogram": {
      if (!message.buckets || typeof message.shotCount !== "number") {
        console.error("No buckets in message: ", message);
        return;
      }
      state = {
        viewType: "histogram",
        buckets: message.buckets as Array<[string, number]>,
        shotCount: message.shotCount,
      };
      break;
    }
    case "estimates":
      {
        const newState: EstimatesState = {
          viewType: "estimates",
          estimatesData: {
            calculating: !!message.calculating,
            estimates: [],
          },
        };
        // Copy over any existing estimates
        if ((state as EstimatesState).estimatesData?.estimates) {
          newState.estimatesData.estimates.push(
            ...(state as EstimatesState).estimatesData.estimates,
          );
        }
        // Append any new estimates
        if (message.estimates) {
          if (Array.isArray(message.estimates)) {
            newState.estimatesData.estimates.push(...message.estimates);
          } else {
            newState.estimatesData.estimates.push(message.estimates);
          }
        }
        state = newState;
      }
      break;
    case "help":
      state = helpState;
      break;
    case "circuit":
      {
        state = {
          viewType: "circuit",
          ...message,
        };
      }
      break;
    case "showDocumentationCommand":
      {
        state = {
          viewType: "documentation",
          fragmentsToRender: message.fragmentsToRender,
        };
      }
      break;
    case "copilot":
      state = {
        viewType: "copilot",
        qas: [{ request: "", response: "AMA!" }],
        inProgress: false,
      };
      break;
    case "copilotResponse":
      {
        if (state.viewType !== "copilot") {
          console.error("Received copilot response in wrong state", state);
          return;
        }
        // TODO: Even with instructions in the context, Copilot keeps using \( and \) for LaTeX
        let cleanedResponse = message.response;
        cleanedResponse = cleanedResponse.replace(/(\\\()|(\\\))/g, "$");
        cleanedResponse = cleanedResponse.replace(/(\\\[)|(\\\])/g, "$$");

        state.qas.push({
          request: message.request ?? "",
          response: cleanedResponse,
        });
      }
      break;
    case "copilotResponseHistogram":
      {
        if (state.viewType !== "copilot") {
          console.error(
            "Received copilot response histogram in wrong state",
            state,
          );
          return;
        }
        if (!message.buckets || typeof message.shotCount !== "number") {
          console.error("No buckets in message: ", message);
          return;
        }
        const buckets = message.buckets as Array<[string, number]>;
        const histogram = JSON.stringify({
          buckets: buckets,
          shotCount: message.shotCount,
        });
        state.qas.push({
          request: "",
          response: "```widget\nHistogram\n" + histogram,
        });
      }
      break;
    case "copilotResponseDone":
      if (state.viewType !== "copilot") {
        console.error("Received copilot response done in wrong state", state);
        return;
      } else {
        state.qas.push({ request: "", response: "\n\n---\n\n" });
        state.inProgress = false;
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

  vscodeApi.setState(state);
  render(<App state={state} />, document.body);
}

function onRowDeleted(rowId: string) {
  // Clone all the state to a new object
  const newState: State = JSON.parse(JSON.stringify(state));

  // Splice out the estimate that was deleted
  const estimates = (newState as EstimatesState).estimatesData.estimates;
  const index = estimates.findIndex(
    (estimate) => estimate.jobParams.runName === rowId,
  );
  if (index >= 0) {
    estimates.splice(index, 1);
  }
  state = newState;

  vscodeApi.setState(state);
  render(<App state={state} />, document.body);
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function copilotRequest() {
  const questionText = document.querySelector(
    "#copilotQuestion",
  ) as HTMLInputElement;
  vscodeApi.postMessage({
    command: "copilotRequest",
    request: questionText.value,
  });
  (state as CopilotState).qas.push({
    request: questionText.value,
    response: "",
  });
  (state as CopilotState).inProgress = true;
  questionText.value = "";
  render(<App state={state} />, document.body);
}

type QA = {
  request: string;
  response: string;
};

function App({ state }: { state: State }) {
  const onFilter = () => undefined;

  switch (state.viewType) {
    case "loading":
      return <div>Loading...</div>;
    case "histogram":
      return (
        <Histogram
          data={new Map(state.buckets)}
          shotCount={state.shotCount}
          filter=""
          onFilter={onFilter}
          shotsHeader={true}
        ></Histogram>
      );
    case "estimates":
      return (
        <EstimatesPanel
          calculating={state.estimatesData.calculating}
          estimatesData={state.estimatesData.estimates}
          onRowDeleted={onRowDeleted}
          colors={[]}
          runNames={[]}
        />
      );
    case "circuit":
      return <CircuitPanel {...state.props}></CircuitPanel>;
    case "help":
      return <HelpPage />;
    case "documentation":
      // Ideally we'd have this on all web views, but it makes the font a little
      // too large in the others right now. Something to unify later.
      document.body.classList.add("markdown-body");
      document.body.style.fontSize = "0.8em";
      return <DocumentationView fragmentsToRender={state.fragmentsToRender} />;
    case "copilot": {
      const hrRef = useRef<HTMLHRElement>(null);
      useEffect(() => {
        hrRef.current?.scrollIntoView(false);
      });
      return (
        <div style="max-width: 800px; font-size: 10pt; line-height: 1.5;">
          <h2 style="margin-top: 0">Welcome to Quantum Copilot</h2>
          {(state as CopilotState).qas.map((qa) => (
            <Response request={qa.request} response={qa.response} />
          ))}
          <br />
          <textarea
            style="width: 90vw; min-height: 32px; max-height: 128px;"
            type="text"
            placeholder="Ask your question here"
            id="copilotQuestion"
          />
          <br />
          <button
            style="margin-top: 8px; margin-bottom: 12px; padding: 4px;"
            onClick={copilotRequest}
          >
            {state.inProgress ? "Cancel" : "Ask Copilot"}
          </button>
          <div style="height: 8px" ref={hrRef} />
        </div>
      );
    }
    default:
      console.error("Unknown view type in state", state);
      return <div>Loading error</div>;
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
    let requestBox = {};
    if (props.request) {
      requestBox = (
        <div class="requestBox">
          <Markdown markdown={props.request} />
        </div>
      );
    }
    return (
      <div>
        {requestBox}
        <div class="responseBox">
          {parts.map((part) => {
            if (part.startsWith("```widget\nHistogram\n")) {
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
            }
            return <Markdown markdown={part}></Markdown>;
          })}
        </div>
      </div>
    );
  }
}
