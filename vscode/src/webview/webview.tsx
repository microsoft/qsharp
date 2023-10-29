// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="@types/vscode-webview"/>

console.info("Loading webview");

const vscodeApi = acquireVsCodeApi();

import { render } from "preact";
import { Histogram } from "./histogram";
import { Estimates } from "./estimates";

window.addEventListener("message", onMessage);
window.addEventListener("load", main);

let histogramData: Map<string, number> = new Map();
let shotCount = 0;
let showEstimates = false;

function main() {
  console.info("WebView loaded. Posting 'ready'");
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
        ></Histogram>
      ) : null}
      <br />
      {showEstimates ? <Estimates /> : null}
    </>
  );
}
