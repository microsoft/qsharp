// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  getLanguageService,
  getLibrarySourceContent,
  loadWasmModule,
  log,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import * as vscode from "vscode";
import {
  isQsharpDocument,
  isQsharpNotebookCell,
  qsharpDocumentFilter,
} from "./common.js";
import { createCompletionItemProvider } from "./completion.js";
import { activateDebugger } from "./debugger/activate.js";
import { createDefinitionProvider } from "./definition.js";
import { startCheckingQSharp } from "./diagnostics.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookHandlers } from "./notebook.js";
import { setupWorkspaces } from "./azure/workspace.js";
import { initCodegen } from "./qirGeneration.js";
import { activateTargetProfileStatusBarItem } from "./statusbar.js";

let extensionPath: vscode.Uri;

export function getResourcePath(...parts: string[]) {
  return vscode.Uri.joinPath(extensionPath, "resources", ...parts);
}

export async function activate(context: vscode.ExtensionContext) {
  initializeLogger();
  log.info("Q# extension activating.");
  extensionPath = context.extensionUri;

  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(
      qsharpLibraryUriScheme,
      new QsTextDocumentContentProvider()
    )
  );

  context.subscriptions.push(...activateTargetProfileStatusBarItem());

  context.subscriptions.push(
    ...(await activateLanguageService(context.extensionUri))
  );

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  setupWorkspaces(context);
  initCodegen(context);
  activateDebugger(context);

  log.info("Q# extension activated.");
}

function initializeLogger() {
  const output = vscode.window.createOutputChannel("Q#", { log: true });

  // Override the global logger with functions that write to the output channel
  log.error = output.error;
  log.warn = output.warn;
  log.info = output.info;
  log.debug = output.debug;
  log.trace = output.trace;

  // The numerical log levels for VS Code and qsharp don't match.
  function mapLogLevel(logLevel: vscode.LogLevel) {
    switch (logLevel) {
      case vscode.LogLevel.Off:
        return "off";
      case vscode.LogLevel.Trace:
        return "trace";
      case vscode.LogLevel.Debug:
        return "debug";
      case vscode.LogLevel.Info:
        return "info";
      case vscode.LogLevel.Warning:
        return "warn";
      case vscode.LogLevel.Error:
        return "error";
    }
  }

  log.setLogLevel(mapLogLevel(output.logLevel));
  output.onDidChangeLogLevel((level) => {
    log.setLogLevel(mapLogLevel(level));
  });
}

function registerDocumentUpdateHandlers(languageService: ILanguageService) {
  vscode.workspace.textDocuments.forEach((document) => {
    updateIfQsharpDocument(document);
  });

  const subscriptions = [];
  subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      updateIfQsharpDocument(document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((evt) => {
      updateIfQsharpDocument(evt.document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      if (isQsharpDocument(document) && !isQsharpNotebookCell(document)) {
        // Notebook cells don't currently support the language service.
        languageService.closeDocument(document.uri.toString());
      }
    })
  );

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (isQsharpDocument(document) && !isQsharpNotebookCell(document)) {
      // Notebook cells don't currently support the language service.
      languageService.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText()
      );
    }
  }

  return subscriptions;
}

async function activateLanguageService(extensionUri: vscode.Uri) {
  const subscriptions: vscode.Disposable[] = [];

  const languageService = await loadLanguageService(extensionUri);

  // synchronize document contents
  subscriptions.push(...registerDocumentUpdateHandlers(languageService));

  // synchronize configuration
  subscriptions.push(registerConfigurationChangeHandlers(languageService));

  // diagnostics
  subscriptions.push(...startCheckingQSharp(languageService));

  // completions
  subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      qsharpDocumentFilter,
      createCompletionItemProvider(languageService),
      "."
    )
  );

  // hover
  subscriptions.push(
    vscode.languages.registerHoverProvider(
      qsharpDocumentFilter,
      createHoverProvider(languageService)
    )
  );

  // go to def
  subscriptions.push(
    vscode.languages.registerDefinitionProvider(
      qsharpDocumentFilter,
      createDefinitionProvider(languageService)
    )
  );

  return subscriptions;
}

async function loadLanguageService(baseUri: vscode.Uri) {
  const wasmUri = vscode.Uri.joinPath(baseUri, "./wasm/qsc_wasm_bg.wasm");
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  return await getLanguageService();
}

function registerConfigurationChangeHandlers(
  languageService: ILanguageService
) {
  return vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration("Q#.targetProfile")) {
      let targetProfile = vscode.workspace
        .getConfiguration("Q#")
        .get<string>("targetProfile", "full");
      log.info(`Target profile changed: ${targetProfile}`);

      switch (targetProfile) {
        case "base":
        case "full":
          break;
        default:
          log.warn(`Invalid value for target profile: ${targetProfile}`);
          targetProfile = "full";
      }

      languageService.updateConfiguration({
        targetProfile: targetProfile as "base" | "full",
      });
    }
  });
}

export class QsTextDocumentContentProvider
  implements vscode.TextDocumentContentProvider
{
  onDidChange?: vscode.Event<vscode.Uri> | undefined;
  provideTextDocumentContent(
    uri: vscode.Uri,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    token: vscode.CancellationToken
  ): vscode.ProviderResult<string> {
    return getLibrarySourceContent(uri.path);
  }
}
