// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { render } from "preact";
import {
  CompilerState,
  QscEventTarget,
  getCompilerWorker,
  loadWasmModule,
  getAllKatas,
  Kata,
  VSDiagnostic,
  log,
  LogLevel,
  samples,
  getLanguageServiceWorker,
  ILanguageService,
} from "qsharp";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { OutputTabs } from "./tabs.js";
import { useState } from "preact/hooks";
import { Kata as Katas } from "./kata.js";
import { compressedBase64ToCode } from "./utils.js";

export type ActiveTab = "results-tab" | "hir-tab" | "logs-tab";

// Configure any logging as early as possible
const logLevelUri = new URLSearchParams(window.location.search).get("logLevel");
if (logLevelUri) log.setLogLevel(logLevelUri as LogLevel);

const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const compilerWorkerPath = basePath + "libs/compiler-worker.js";
const languageServiceWorkerPath = basePath + "libs/language-service-worker.js";

declare global {
  const MathJax: { typeset: () => void };
}

const wasmPromise = loadWasmModule(modulePath); // Start loading but don't wait on it

function createCompiler(onStateChange: (val: CompilerState) => void) {
  log.info("In createCompiler");
  const compiler = getCompilerWorker(compilerWorkerPath);
  compiler.onstatechange = onStateChange;
  return compiler;
}

function App(props: { katas: Kata[]; linkedCode?: string }) {
  const [compilerState, setCompilerState] = useState<CompilerState>("idle");
  const [compiler, setCompiler] = useState(() =>
    createCompiler(setCompilerState)
  );
  const [evtTarget] = useState(new QscEventTarget(true));

  const languageService = getLanguageServiceWorker(languageServiceWorkerPath);
  registerMonacoLanguageServiceProviders(languageService);

  const [currentNavItem, setCurrentNavItem] = useState(
    props.linkedCode ? "linked" : "Minimal"
  );
  const [shotError, setShotError] = useState<VSDiagnostic | undefined>(
    undefined
  );

  const [hir, setHir] = useState<string>("");
  const [activeTab, setActiveTab] = useState<ActiveTab>("results-tab");

  const onRestartCompiler = () => {
    compiler.terminate();
    const newCompiler = createCompiler(setCompilerState);
    setCompiler(newCompiler);
    setCompilerState("idle");
  };

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
            compiler={compiler}
            compilerState={compilerState}
            onRestartCompiler={onRestartCompiler}
            evtTarget={evtTarget}
            defaultShots={defaultShots}
            showShots={true}
            showExpr={true}
            shotError={shotError}
            setHir={setHir}
            activeTab={activeTab}
            languageService={languageService}
          ></Editor>
          <OutputTabs
            evtTarget={evtTarget}
            showPanel={true}
            onShotError={onShotError}
            hir={hir}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          ></OutputTabs>
        </>
      ) : (
        <Katas
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          kata={activeKata!}
          compiler={compiler}
          compilerState={compilerState}
          onRestartCompiler={onRestartCompiler}
          languageService={languageService}
        ></Katas>
      )}
      <div id="popup"></div>
    </>
  );
}

// Called once Monaco is ready
async function loaded() {
  await wasmPromise; // Block until the wasm module is loaded
  const katas = await getAllKatas();

  // If URL is a sharing link, populate the editor with the code from the link.
  // Otherwise, populate with sample code.
  let linkedCode: string | undefined;
  const paramCode = new URLSearchParams(window.location.search).get("code");
  if (paramCode) {
    try {
      const base64code = decodeURIComponent(paramCode);
      linkedCode = await compressedBase64ToCode(base64code);
    } catch {
      linkedCode = "// Unable to decode the code in the URL\n";
    }
  }

  render(<App katas={katas} linkedCode={linkedCode}></App>, document.body);
}

function registerMonacoLanguageServiceProviders(
  languageService: ILanguageService
) {
  monaco.languages.registerCompletionItemProvider("qsharp", {
    // @ts-expect-error - Monaco's types expect range to be defined,
    // but it's actually optional and the default behavior is better
    provideCompletionItems: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position
    ) => {
      const completions = await languageService.getCompletions(
        model.uri.toString(),
        model.getOffsetAt(position)
      );
      return {
        suggestions: completions.items.map((i) => {
          let kind;
          switch (i.kind) {
            case "function":
              kind = monaco.languages.CompletionItemKind.Function;
              break;
            case "module":
              kind = monaco.languages.CompletionItemKind.Module;
              break;
            case "keyword":
              kind = monaco.languages.CompletionItemKind.Keyword;
              break;
            case "issue":
              kind = monaco.languages.CompletionItemKind.Issue;
              break;
          }
          return {
            label: i.label,
            kind: kind,
            insertText: i.label,
            range: undefined,
          };
        }),
      };
    },
  });

  monaco.languages.registerHoverProvider("qsharp", {
    provideHover: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position
    ) => {
      const hover = await languageService.getHover(
        model.uri.toString(),
        model.getOffsetAt(position)
      );

      if (hover) {
        const start = model.getPositionAt(hover.span.start);
        const end = model.getPositionAt(hover.span.end);

        return {
          contents: [{ value: hover.contents }],
          range: {
            startLineNumber: start.lineNumber,
            startColumn: start.column,
            endLineNumber: end.lineNumber,
            endColumn: end.column,
          },
        };
      }
      return null;
    },
  });

  monaco.languages.registerDefinitionProvider("qsharp", {
    provideDefinition: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position
    ) => {
      const definition = await languageService.getDefinition(
        model.uri.toString(),
        model.getOffsetAt(position)
      );

      if (!definition) return null;
      const uri = monaco.Uri.parse(definition.source);
      const definitionPosition =
        uri.toString() === model.uri.toString()
          ? model.getPositionAt(definition.offset)
          : { lineNumber: 1, column: 1 };
      return {
        uri,
        range: {
          startLineNumber: definitionPosition.lineNumber,
          startColumn: definitionPosition.column,
          endLineNumber: definitionPosition.lineNumber,
          endColumn: definitionPosition.column + 1,
        },
      };
    },
  });
}

// Monaco provides the 'require' global for loading modules.
declare const require: {
  config: (settings: object) => void;
  (base: string[], onready: () => void): void;
};
require.config({ paths: { vs: monacoPath } });
require(["vs/editor/editor.main"], loaded);
