// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Use esbuild to bundle and copy the CSS files to the output directory.
import "modern-normalize/modern-normalize.css";

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
} from "qsharp-lang";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { OutputTabs } from "./tabs.js";
import { useState } from "preact/hooks";
import { Kata as Katas } from "./kata.js";
import { compressedBase64ToCode } from "./utils.js";

export type ActiveTab = "results-tab" | "hir-tab" | "logs-tab";

const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const compilerWorkerPath = basePath + "libs/compiler-worker.js";
const languageServiceWorkerPath = basePath + "libs/language-service-worker.js";

declare global {
  const MathJax: { typeset: () => void };
}

function telemetryHandler({ id, data }: { id: string; data?: any }) {
  // NOTE: This is for demo purposes. Wire up to the real telemetry library.
  console.log(`Received telemetry event: "%s" with payload: %o`, id, data);
}

function createCompiler(onStateChange: (val: CompilerState) => void) {
  log.info("In createCompiler");
  const compiler = getCompilerWorker(compilerWorkerPath);
  compiler.onstatechange = onStateChange;
  return compiler;
}

function App(props: { katas: Kata[]; linkedCode?: string }) {
  const [compilerState, setCompilerState] = useState<CompilerState>("idle");
  const [compiler, setCompiler] = useState(() =>
    createCompiler(setCompilerState),
  );
  const [evtTarget] = useState(() => new QscEventTarget(true));

  const [languageService] = useState(() => {
    const languageService = getLanguageServiceWorker(languageServiceWorkerPath);
    registerMonacoLanguageServiceProviders(languageService);
    return languageService;
  });

  const [currentNavItem, setCurrentNavItem] = useState(
    props.linkedCode ? "linked" : "Minimal",
  );
  const [shotError, setShotError] = useState<VSDiagnostic | undefined>(
    undefined,
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

  return (
    <>
      <header class="page-header">Q# playground</header>
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
            onShotError={(diag?: VSDiagnostic) => setShotError(diag)}
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
  // Configure any logging as early as possible
  const logLevelUri = new URLSearchParams(window.location.search).get(
    "logLevel",
  );
  if (logLevelUri) {
    log.setLogLevel(logLevelUri as LogLevel);
  } else {
    log.setLogLevel("error");
  }
  log.setTelemetryCollector(telemetryHandler);

  await loadWasmModule(modulePath);

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
  languageService: ILanguageService,
) {
  monaco.languages.registerCompletionItemProvider("qsharp", {
    // @ts-expect-error - Monaco's types expect range to be defined,
    // but it's actually optional and the default behavior is better
    provideCompletionItems: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ) => {
      const completions = await languageService.getCompletions(
        model.uri.toString(),
        model.getOffsetAt(position),
      );
      return {
        suggestions: completions.items.map((i) => {
          let kind;
          switch (i.kind) {
            case "function":
              kind = monaco.languages.CompletionItemKind.Function;
              break;
            case "interface":
              kind = monaco.languages.CompletionItemKind.Interface;
              break;
            case "keyword":
              kind = monaco.languages.CompletionItemKind.Keyword;
              break;
            case "module":
              kind = monaco.languages.CompletionItemKind.Module;
              break;
            case "property":
              kind = monaco.languages.CompletionItemKind.Property;
              break;
          }
          return {
            label: i.label,
            kind: kind,
            insertText: i.label,
            sortText: i.sortText,
            detail: i.detail,
            additionalTextEdits: i.additionalTextEdits?.map((edit) => {
              const start = model.getPositionAt(edit.range.start);
              const end = model.getPositionAt(edit.range.end);
              const textEdit: monaco.languages.TextEdit = {
                range: new monaco.Range(
                  start.lineNumber,
                  start.column,
                  end.lineNumber,
                  end.column,
                ),
                text: edit.newText,
              };
              return textEdit;
            }),
            range: undefined,
          };
        }),
      };
    },
    triggerCharacters: ["@"], // for attribute completions
  });

  monaco.languages.registerHoverProvider("qsharp", {
    provideHover: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ) => {
      const hover = await languageService.getHover(
        model.uri.toString(),
        model.getOffsetAt(position),
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
      position: monaco.Position,
    ) => {
      const definition = await languageService.getDefinition(
        model.uri.toString(),
        model.getOffsetAt(position),
      );
      if (!definition) return null;
      const uri = monaco.Uri.parse(definition.source);
      if (uri.toString() !== model.uri.toString()) return null;
      const defStartPosition = model.getPositionAt(definition.span.start);
      const defEndPosition = model.getPositionAt(definition.span.end);
      return {
        uri,
        range: {
          startLineNumber: defStartPosition.lineNumber,
          startColumn: defStartPosition.column,
          endLineNumber: defEndPosition.lineNumber,
          endColumn: defEndPosition.column,
        },
      };
    },
  });

  monaco.languages.registerReferenceProvider("qsharp", {
    provideReferences: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
      context: monaco.languages.ReferenceContext,
    ) => {
      const lsReferences = await languageService.getReferences(
        model.uri.toString(),
        model.getOffsetAt(position),
        context.includeDeclaration,
      );
      if (!lsReferences) return [];
      const references: monaco.languages.Location[] = [];
      for (const reference of lsReferences) {
        const uri = monaco.Uri.parse(reference.source);
        // the playground doesn't support sources other than the current source
        if (uri.toString() == model.uri.toString()) {
          const refStartPosition = model.getPositionAt(reference.span.start);
          const refEndPosition = model.getPositionAt(reference.span.end);
          references.push({
            uri,
            range: {
              startLineNumber: refStartPosition.lineNumber,
              startColumn: refStartPosition.column,
              endLineNumber: refEndPosition.lineNumber,
              endColumn: refEndPosition.column,
            },
          });
        }
      }
      return references;
    },
  });

  monaco.languages.registerSignatureHelpProvider("qsharp", {
    signatureHelpTriggerCharacters: ["(", ","],
    provideSignatureHelp: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ) => {
      const sigHelpLs = await languageService.getSignatureHelp(
        model.uri.toString(),
        model.getOffsetAt(position),
      );
      if (!sigHelpLs) return null;
      return {
        // eslint-disable-next-line @typescript-eslint/no-empty-function
        dispose: () => {},
        value: {
          activeParameter: sigHelpLs.activeParameter,
          activeSignature: sigHelpLs.activeSignature,
          signatures: sigHelpLs.signatures.map((sig) => {
            return {
              label: sig.label,
              documentation: {
                value: sig.documentation,
              } as monaco.IMarkdownString,
              parameters: sig.parameters.map((param) => {
                return {
                  label: [param.label.start, param.label.end],
                  documentation: {
                    value: param.documentation,
                  } as monaco.IMarkdownString,
                };
              }),
            };
          }),
        },
      };
    },
  });

  monaco.languages.registerRenameProvider("qsharp", {
    provideRenameEdits: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
      newName: string,
    ) => {
      const rename = await languageService.getRename(
        model.uri.toString(),
        model.getOffsetAt(position),
        newName,
      );
      if (!rename) return null;

      const edits = rename.changes.flatMap(([uri, edits]) => {
        return edits.map((edit) => {
          const start = model.getPositionAt(edit.range.start);
          const end = model.getPositionAt(edit.range.end);
          const textEdit: monaco.languages.TextEdit = {
            range: new monaco.Range(
              start.lineNumber,
              start.column,
              end.lineNumber,
              end.column,
            ),
            text: edit.newText,
          };
          return {
            resource: monaco.Uri.parse(uri),
            textEdit: textEdit,
          } as monaco.languages.IWorkspaceTextEdit;
        });
      });
      return { edits: edits } as monaco.languages.WorkspaceEdit;
    },
    resolveRenameLocation: async (
      model: monaco.editor.ITextModel,
      position: monaco.Position,
    ) => {
      const prepareRename = await languageService.prepareRename(
        model.uri.toString(),
        model.getOffsetAt(position),
      );
      if (prepareRename) {
        const start = model.getPositionAt(prepareRename.range.start);
        const end = model.getPositionAt(prepareRename.range.end);
        return {
          range: new monaco.Range(
            start.lineNumber,
            start.column,
            end.lineNumber,
            end.column,
          ),
          text: prepareRename.newText,
        } as monaco.languages.RenameLocation;
      } else {
        return {
          rejectReason: "Rename is unavailable at this location.",
        } as monaco.languages.RenameLocation & monaco.languages.Rejection;
      }
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
