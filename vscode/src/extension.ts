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
    // Of course the uri should come from the event
    const document = vscode.window.activeTextEditor?.document;
    if (document) {
      diagCollection.clear();
      diagCollection.set(
        document.uri,
        diagnostics.map(
          (d) =>
            new vscode.Diagnostic(
              new vscode.Range(
                document.positionAt(d.start_pos),
                document.positionAt(d.end_pos)
              ),
              d.message,
              d.severity
            )
        )
      );
    }
  });

  // notebooks
  registerQSharpNotebookHandlers(context);
}

function registerDocumentUpdateHandlers(
  output: vscode.OutputChannel,
  compiler: Awaited<ReturnType<typeof getCompiler>>
) {
  const document = vscode.window.activeTextEditor?.document;
  if (document && vscode.languages.match("qsharp", document)) {
    output.appendLine(`Active document: ${document.uri.toString()}`);
    compiler.updateCode(document.uri.toString(), document.getText());
  }

  vscode.window.onDidChangeActiveTextEditor((editor) => {
    const document = editor?.document;
    if (document && vscode.languages.match("qsharp", document)) {
      output.appendLine(`Active document changed: ${document.uri.toString()}`);
      compiler.updateCode(document.uri.toString(), document.getText());
    }
  });

  vscode.workspace.onDidChangeTextDocument((evt) => {
    if (vscode.languages.match("qsharp", evt.document)) {
      compiler.updateCode(evt.document.uri.toString(), evt.document.getText());
    }
  });
}
