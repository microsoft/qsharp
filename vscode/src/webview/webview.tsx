// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import { EstimatesPanel, Histogram, type ReData } from "qsharp-lang/ux";
import { HelpPage } from "./help";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - there are no types for this
import mk from "@vscode/markdown-it-katex";
import markdownIt from "markdown-it";
const md = markdownIt();
md.use(mk);

function markdownRenderer(input: string) {
  // For some reason all the escape characters are doubled in the estimate data sample input
  return md.render(input.replace(/\\\\/g, "\\"));
}

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

type State =
  | { viewType: "loading" }
  | { viewType: "help" }
  | HistogramState
  | EstimatesState;
const loadingState: State = { viewType: "loading" };
const helpState: State = { viewType: "help" };
let state: State = loadingState;

function main() {
  state = (vscodeApi.getState() as any) || loadingState;
  render(<App state={state} />, document.body);
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
          renderer={markdownRenderer}
          onRowDeleted={onRowDeleted}
          colors={[]}
          runNames={[]}
        />
      );
    case "help":
      return <HelpPage />;
    default:
      console.error("Unknown view type in state", state);
      return <div>Loading error</div>;
  }
}
