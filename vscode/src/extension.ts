import * as vscode from "vscode";
import { QscEventTarget, getCompiler, loadWasmModule } from "qsharp";
import { createCompletionItemProvider } from "./completion.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookHandlers } from "./notebooks.js";
import { createDefinitionProvider } from "./definition.js";

export async function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Q#");
  output.appendLine("Q# extension activated.");

  // load the compiler
  const wasmUri = vscode.Uri.joinPath(
    context.extensionUri,
    "./wasm/qsc_wasm_bg.wasm"
  );
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  const evtTarget = new QscEventTarget(false);
  const compiler = await getCompiler(evtTarget);

  // send document updates
  vscode.workspace.onDidChangeTextDocument((evt) => {
    if (vscode.languages.match("qsharp", evt.document)) {
      compiler.updateCode(evt.document.uri.toString(), evt.document.getText());
    }
  });

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
