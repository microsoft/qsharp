// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService, getLanguageService, loadWasmModule } from "qsharp";
import * as vscode from "vscode";
import { createCompletionItemProvider } from "./completion.js";
import { createDefinitionProvider } from "./definition.js";
import { startCheckingQSharp } from "./diagnostics.js";
import { createHoverProvider } from "./hover.js";
import {
  qsharpDocumentFilter,
  qsharpNotebookCellDocumentFilter,
} from "../common.js";

export async function activateLanguageService(extensionUri: vscode.Uri) {
  const subscriptions = [];

  const languageService = await loadLanguageService(extensionUri);

  subscriptions.push(...registerDocumentUpdateHandlers(languageService));

  // diagnostics
  subscriptions.push(startCheckingQSharp(languageService));

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
      "qsharp",
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
      if (
        vscode.languages.match(qsharpDocumentFilter, document) &&
        !vscode.languages.match(qsharpNotebookCellDocumentFilter, document)
      ) {
        // Notebook cells don't currently support the language service.
        languageService.closeDocument(document.uri.toString());
      }
    })
  );

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (
      vscode.languages.match(qsharpDocumentFilter, document) &&
      !vscode.languages.match(qsharpNotebookCellDocumentFilter, document)
    ) {
      // Notebook cells don't currently support the language service.
      languageService.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText(),
        true // PackageType "exe"
      );
    }
  }

  return subscriptions;
}
