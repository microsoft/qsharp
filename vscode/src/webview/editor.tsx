// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import DOMPurify from "dompurify";
import { CircuitPanel, CircuitProps } from "qsharp-lang/ux";
import { setThemeStylesheet } from "./theme";

window.addEventListener("message", onMessage);
window.addEventListener("load", main);

type CircuitState = {
  viewType: "circuit";
  props: CircuitProps;
};

type State = { viewType: "loading" } | CircuitState;
const loadingState: State = { viewType: "loading" };
let state: State = loadingState;

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
    case "error": {
      const sanitizedMessage = DOMPurify.sanitize(message.props.message);
      const sanitizedTitle = DOMPurify.sanitize(message.props.title);
      document.body.innerHTML = `
        <div class="error">
          <h1>${sanitizedTitle}</h1>
          <p>${sanitizedMessage}</p>
        </div>
      `;
      return;
    }
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

function runCircuit(circuit: any) {
  vscodeApi.postMessage({
    command: "run",
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
          runCallback={runCircuit}
        ></CircuitPanel>
      );
    default:
      console.error("Unknown view type in state", state);
      return <div>Loading error</div>;
  }
}
