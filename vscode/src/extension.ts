import * as vscode from "vscode";
import { QscEventTarget, getCompiler, loadWasmModule } from "qsharp";
import { createCompletionItemProvider } from "./completion.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookHandlers } from "./notebooks.js";
import { createDefinitionProvider } from "./definition.js";

export async function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Q#");
  output.appendLine("Q# extension activated.");

  // Override the global logger with functions that write to the output channel
  global.qscLog.error = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.warn = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.info = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.debug = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };
  global.qscLog.trace = (...args: unknown[]) => {
    output.appendLine(
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ")
    );
  };

  // load the compiler
  const wasmUri = vscode.Uri.joinPath(
    context.extensionUri,
    "./wasm/qsc_wasm_bg.wasm"
  );
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  const evtTarget = new QscEventTarget(false);
  const compiler = await getCompiler(evtTarget);

  registerDocumentUpdateHandlers(output, compiler);

  // completions
  vscode.languages.registerCompletionItemProvider(
    "qsharp",
    createCompletionItemProvider(compiler),
    "."
  );

  // hover
  vscode.languages.registerHoverProvider(
    "qsharp",
    createHoverProvider(compiler)
  );

  // go to def
  vscode.languages.registerDefinitionProvider(
    "qsharp",
    createDefinitionProvider(compiler)
  );

  const diagCollection = vscode.languages.createDiagnosticCollection("qsharp");
  evtTarget.addEventListener("diagnostics", (evt) => {
    const diagnostics = evt.detail;

    const getPosition = (offset: number) => {
      // We need the document here to be able to map offsets to line/column positions.
      // The document may not be available if this event is to clear diagnostics
      // for an already-closed document from the problems list.
      // Note: This mapping will break down if we ever send diagnostics for closed files.
      const document = vscode.workspace.textDocuments.filter(
        (doc) => doc.uri.toString() === diagnostics.uri
      )[0];
      return document.positionAt(offset);
    };

    diagCollection.set(
      vscode.Uri.parse(evt.detail.uri),
      diagnostics.diagnostics.map(
        (d) =>
          new vscode.Diagnostic(
            new vscode.Range(getPosition(d.start_pos), getPosition(d.end_pos)),
            d.message,
            d.severity
          )
      )
    );
  });

  // notebooks
  registerQSharpNotebookHandlers(context);
}

function registerDocumentUpdateHandlers(
  output: vscode.OutputChannel,
  compiler: Awaited<ReturnType<typeof getCompiler>>
) {
  vscode.workspace.textDocuments.forEach((document) => {
    updateIfQsharpDocument(document);
  });

  vscode.workspace.onDidOpenTextDocument((document) => {
    updateIfQsharpDocument(document);
  });

  vscode.workspace.onDidChangeTextDocument((evt) => {
    updateIfQsharpDocument(evt.document);
  });

  vscode.workspace.onDidCloseTextDocument((document) => {
    if (vscode.languages.match("qsharp", document)) {
      compiler.closeDocument(document.uri.toString());
    }
  });

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (vscode.languages.match("qsharp", document)) {
      compiler.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText()
      );
    }
  }
}
