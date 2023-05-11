// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { render } from "preact";
import {
  ICompilerWorker,
  QscEventTarget,
  getCompilerWorker,
  loadWasmModule,
  getAllKatas,
  Kata,
  VSDiagnostic,
  samples,
} from "qsharp";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { Tabs } from "./results.js";
import { useState } from "preact/hooks";
import { Kata as Katas } from "./kata.js";
import { base64ToCode } from "./utils.js";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const workerPath = basePath + "libs/worker.js";

declare global {
  const MathJax: { typeset: () => void };
}

const wasmPromise = loadWasmModule(modulePath); // Start loading but don't wait on it

function App(props: {
  compiler: ICompilerWorker;
  evtTarget: QscEventTarget;
  katas: Kata[];
  linkedCode?: string;
}) {
  const [currentNavItem, setCurrentNavItem] = useState(
    props.linkedCode ? "linked" : "Minimal"
  );
  const [shotError, setShotError] = useState<VSDiagnostic | undefined>(
    undefined
  );

  const kataTitles = props.katas.map((elem) => elem.title);
  const sampleTitles = samples.map((sample) => sample.title);

  const sampleCode =
    samples.find((sample) => sample.title === currentNavItem)?.code ||
    props.linkedCode;

  const defaultShots =
    samples.find((sample) => sample.title === currentNavItem)?.shots || 100;

  const activeKata = kataTitles.includes(currentNavItem)
    ? props.katas.find((kata) => kata.title === currentNavItem)
    : undefined;

  function onNavItemSelected(name: string) {
    // If there was a ?code link on the URL before, clear it out
    const params = new URLSearchParams(window.location.search);
    if (params.get("code")) {
      // Get current URL without query parameters to use as the URL
      const newUrl = `${window.location.href.split("?")[0]}`;
      window.history.pushState({}, "", newUrl);
    }
    setCurrentNavItem(name);
  }

  function onShotError(diag?: VSDiagnostic) {
    // TODO: Should this be for katas too and not just the main editor?
    setShotError(diag);
  }

  return (
    <>
      <header class="header">Q# playground</header>
      <Nav
        selected={currentNavItem}
        navSelected={onNavItemSelected}
        katas={kataTitles}
        samples={sampleTitles}
      ></Nav>
      {sampleCode ? (
        <>
          <Editor
            code={sampleCode}
            compiler={props.compiler}
            evtTarget={props.evtTarget}
            defaultShots={defaultShots}
            showShots={true}
            showExpr={false}
            shotError={shotError}
          ></Editor>
          <Tabs
            evtTarget={props.evtTarget}
            showPanel={true}
            onShotError={onShotError}
          ></Tabs>
        </>
      ) : (
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        <Katas kata={activeKata!} compiler={props.compiler}></Katas>
      )}
    </>
  );
}

// Called once Monaco is ready
async function loaded() {
  await wasmPromise; // Block until the wasm module is loaded
  const katas = await getAllKatas();
  const evtHander = new QscEventTarget(true);
  const compiler = await getCompilerWorker(workerPath);

  // If URL is a sharing link, populate the editor with the code from the link.
  // Otherwise, populate with sample code.
  let linkedCode: string | undefined;
  const paramCode = new URLSearchParams(window.location.search).get("code");
  if (paramCode) {
    const base64code = decodeURIComponent(paramCode);
    linkedCode = base64ToCode(base64code);
  }

  render(
    <App
      compiler={compiler}
      evtTarget={evtHander}
      katas={katas}
      linkedCode={linkedCode}
    ></App>,
    document.body
  );
}

// Monaco provides the 'require' global for loading modules.
declare const require: {
  config: (settings: object) => void;
  (base: string[], onready: () => void): void;
};
require.config({ paths: { vs: monacoPath } });
require(["vs/editor/editor.main"], loaded);
