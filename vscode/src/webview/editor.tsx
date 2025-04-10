// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import { CircuitPanel, CircuitProps } from "qsharp-lang/ux";

window.addEventListener("message", onMessage);
window.addEventListener("load", main);

type CircuitState = {
  viewType: "circuit";
  props: CircuitProps;
};

type State = { viewType: "loading" } | CircuitState;
const loadingState: State = { viewType: "loading" };
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
  readFromTextDocument();
}

function onMessage(event: any) {
  const message = event.data;
  if (!message?.command) {
    console.error("Unknown message: ", message);
    return;
  }
  switch (message.command) {
    case "circuit":
      {
        // Check if the received circuit is different from the current state
        if (
          state.viewType === "circuit" &&
          JSON.stringify(state.props.circuit) ===
            JSON.stringify(message.props.circuit)
        ) {
          return;
        }

        state = {
          viewType: "circuit",
          ...message,
        };
      }
      break;
    default:
      console.error("Unknown command: ", message.command);
      return;
  }

  vscodeApi.setState(state);
  render(<App state={state} />, document.body);
}

function readFromTextDocument() {
  vscodeApi.postMessage({ command: "read" });
}

function updateTextDocument(circuit: any) {
  vscodeApi.postMessage({
    command: "update",
    text: JSON.stringify(circuit, null, 2),
  });
}

function App({ state }: { state: State }) {
  switch (state.viewType) {
    case "loading":
      return <div>Loading...</div>;
    case "circuit":
      return (
        <CircuitPanel
          {...state.props}
          isEditable={true}
          editCallback={updateTextDocument}
        ></CircuitPanel>
      );
    default:
      console.error("Unknown view type in state", state);
      return <div>Loading error</div>;
  }
}
