import * as vscode from "vscode";
import { getCompiler, loadWasmModule } from "qsharp";
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
  const compiler = await getCompiler();

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

  // notebooks
  registerQSharpNotebookHandlers(context);
}
