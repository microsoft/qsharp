// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import { Histogram, ReTable, SpaceChart } from "qsharp-lang/ux";
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

let histogramData: Map<string, number> = new Map();
let shotCount = 0;
let showEstimates = false;
let estimatesData: any = undefined;
let showHelp = false;

function main() {
  vscodeApi.postMessage({ command: "ready" });
}

function onMessage(event: any) {
  console.info("WebView got message: ", event.data);
  const message = event.data;
  if (!message?.command) {
    console.error("Unknown message: ", message);
    return;
  }
  switch (message.command) {
    case "update": {
      if (!message.buckets || typeof message.shotCount !== "number") {
        console.error("No buckets in message: ", message);
        return;
      }
      const buckets = message.buckets as Array<[string, number]>;
      histogramData = new Map(buckets);
      shotCount = message.shotCount;
      break;
    }
    case "estimate":
      {
        showEstimates = true;
        estimatesData = message.estimatesData;
      }
      break;
    case "help":
      {
        showHelp = true;
      }
      break;
    default:
      console.error("Unknown command: ", message.command);
      return;
  }
  render(<App />, document.body);
}

function App() {
  const onFilter = () => undefined;

  return (
    <>
      {shotCount ? (
        <Histogram
          data={histogramData}
          shotCount={shotCount}
          filter=""
          onFilter={onFilter}
          shotsHeader={true}
        ></Histogram>
      ) : null}
      {showEstimates ? (
        <>
          <ReTable
            mdRenderer={markdownRenderer}
            estimatesData={estimatesData}
          />
          <h2 style="margin: 24px 8px;">Space diagram</h2>
          <SpaceChart estimatesData={estimatesData} />
        </>
      ) : null}
      {showHelp ? <HelpPage /> : null}
    </>
  );
}
