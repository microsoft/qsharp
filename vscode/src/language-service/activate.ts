// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  getLanguageService,
  loadWasmModule,
  log,
} from "qsharp-lang";
import * as vscode from "vscode";
import {
  isQsharpDocument,
  isQsharpNotebookCell,
  qsharpLanguageId,
} from "../common.js";
import { getTarget } from "../config.js";
import {
  fetchGithubRaw,
  findManifestDirectory,
  listDirectory,
  readFile,
  resolvePath,
} from "../projectSystem.js";
import {
  EventType,
  QsharpDocumentType,
  sendTelemetryEvent,
} from "../telemetry.js";
import { createCodeActionsProvider } from "./codeActions.js";
import { createCodeLensProvider } from "./codeLens.js";
import { createCompletionItemProvider } from "./completion.js";
import { createDefinitionProvider } from "./definition.js";
import { startLanguageServiceDiagnostics } from "./diagnostics.js";
import { createFormattingProvider } from "./format.js";
import { createHoverProvider } from "./hover.js";
import { registerQSharpNotebookCellUpdateHandlers } from "./notebook.js";
import { createReferenceProvider } from "./references.js";
import { createRenameProvider } from "./rename.js";
import { createSignatureHelpProvider } from "./signature.js";

export async function activateLanguageService(extensionUri: vscode.Uri) {
  const subscriptions: vscode.Disposable[] = [];

  const languageService = await loadLanguageService(extensionUri);

  // diagnostics
  subscriptions.push(...startLanguageServiceDiagnostics(languageService));

  // synchronize document contents
  subscriptions.push(...registerDocumentUpdateHandlers(languageService));

  // synchronize notebook cell contents
  subscriptions.push(
    ...registerQSharpNotebookCellUpdateHandlers(languageService),
  );

  // synchronize configuration
  subscriptions.push(registerConfigurationChangeHandlers(languageService));

  // format document
  subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      qsharpLanguageId,
      createFormattingProvider(languageService),
    ),
  );

  // format range
  subscriptions.push(
    vscode.languages.registerDocumentRangeFormattingEditProvider(
      qsharpLanguageId,
      createFormattingProvider(languageService),
    ),
  );

  // completions
  subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      qsharpLanguageId,
      createCompletionItemProvider(languageService),
      // Trigger characters should be kept in sync with the ones in `playground/src/main.tsx`
      "@",
      ".",
    ),
  );

  // hover
  subscriptions.push(
    vscode.languages.registerHoverProvider(
      qsharpLanguageId,
      createHoverProvider(languageService),
    ),
  );

  // go to def
  subscriptions.push(
    vscode.languages.registerDefinitionProvider(
      qsharpLanguageId,
      createDefinitionProvider(languageService),
    ),
  );

  // find references
  subscriptions.push(
    vscode.languages.registerReferenceProvider(
      qsharpLanguageId,
      createReferenceProvider(languageService),
    ),
  );

  // signature help
  subscriptions.push(
    vscode.languages.registerSignatureHelpProvider(
      qsharpLanguageId,
      createSignatureHelpProvider(languageService),
      "(",
      ",",
    ),
  );

  // rename symbol
  subscriptions.push(
    vscode.languages.registerRenameProvider(
      qsharpLanguageId,
      createRenameProvider(languageService),
    ),
  );

  // code lens
  subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      qsharpLanguageId,
      createCodeLensProvider(languageService),
    ),
  );

  subscriptions.push(
    vscode.languages.registerCodeActionsProvider(
      qsharpLanguageId,
      createCodeActionsProvider(languageService),
    ),
  );

  // add the language service dispose handler as well
  subscriptions.push(languageService);

  return subscriptions;
}

async function loadLanguageService(baseUri: vscode.Uri) {
  const start = performance.now();
  const wasmUri = vscode.Uri.joinPath(baseUri, "./wasm/qsc_wasm_bg.wasm");
  const wasmBytes = await vscode.workspace.fs.readFile(wasmUri);
  await loadWasmModule(wasmBytes);
  const languageService = await getLanguageService({
    findManifestDirectory,
    readFile,
    listDirectory,
    resolvePath: async (a, b) => resolvePath(a, b),
    fetchGithub: fetchGithubRaw,
  });
  await updateLanguageServiceProfile(languageService);
  const end = performance.now();
  sendTelemetryEvent(
    EventType.LoadLanguageService,
    {},
    { timeToStartMs: end - start },
  );
  return languageService;
}
function registerDocumentUpdateHandlers(languageService: ILanguageService) {
  vscode.workspace.textDocuments.forEach((document) => {
    updateIfQsharpDocument(document);
  });

  // we manually send an OpenDocument telemetry event if this is a Q# document, because the
  // below subscriptions won't fire for documents that are already open when the extension is activated
  vscode.workspace.textDocuments.forEach((document) => {
    if (isQsharpDocument(document)) {
      const documentType = isQsharpNotebookCell(document)
        ? QsharpDocumentType.JupyterCell
        : QsharpDocumentType.Qsharp;
      sendTelemetryEvent(
        EventType.OpenedDocument,
        { documentType },
        { linesOfCode: document.lineCount },
      );
    }
  });

  const subscriptions = [];
  subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      const documentType = isQsharpNotebookCell(document)
        ? QsharpDocumentType.JupyterCell
        : isQsharpDocument(document)
          ? QsharpDocumentType.Qsharp
          : QsharpDocumentType.Other;
      if (documentType !== QsharpDocumentType.Other) {
        sendTelemetryEvent(
          EventType.OpenedDocument,
          { documentType },
          { linesOfCode: document.lineCount },
        );
      }
      updateIfQsharpDocument(document);
    }),
  );

  subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((evt) => {
      updateIfQsharpDocument(evt.document);
    }),
  );

  subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      if (isQsharpDocument(document) && !isQsharpNotebookCell(document)) {
        languageService.closeDocument(document.uri.toString());
      }
    }),
  );

  // Watch manifest changes and update each document in the same project as the manifest.
  subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((manifest) => {
      updateProjectDocuments(manifest.uri);
    }),
  );

  // Trigger an update on all .qs child documents when their manifest is deleted,
  // so that they can get reparented to single-file-projects.
  subscriptions.push(
    vscode.workspace.onDidDeleteFiles((event) => {
      event.files.forEach((uri) => {
        updateProjectDocuments(uri);
      });
    }),
  );

  // Checks if the URI belongs to a qsharp manifest, and updates all
  // open documents in the same project as the manifest.
  function updateProjectDocuments(manifest: vscode.Uri) {
    if (manifest.scheme === "file" && manifest.fsPath.endsWith("qsharp.json")) {
      const project_folder = manifest.fsPath.slice(
        0,
        manifest.fsPath.length - "qsharp.json".length,
      );
      vscode.workspace.textDocuments.forEach((document) => {
        if (
          !document.isClosed &&
          // Check that the document is on the same project as the manifest.
          document.fileName.startsWith(project_folder)
        ) {
          updateIfQsharpDocument(document);
        }
      });
    }
  }

  function updateIfQsharpDocument(document: vscode.TextDocument) {
    if (isQsharpDocument(document) && !isQsharpNotebookCell(document)) {
      // Regular (not notebook) Q# document.
      languageService.updateDocument(
        document.uri.toString(),
        document.version,
        document.getText(),
      );
    }
  }

  return subscriptions;
}

function registerConfigurationChangeHandlers(
  languageService: ILanguageService,
) {
  return vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration("Q#.qir.targetProfile")) {
      updateLanguageServiceProfile(languageService);
    }
  });
}

async function updateLanguageServiceProfile(languageService: ILanguageService) {
  const targetProfile = getTarget();

  switch (targetProfile) {
    case "base":
    case "adaptive_ri":
    case "unrestricted":
      break;
    default:
      log.warn(`Invalid value for target profile: ${targetProfile}`);
  }
  log.debug("Target profile set to: " + targetProfile);

  languageService.updateConfiguration({
    targetProfile: targetProfile,
    lints: [{ lint: "needlessOperation", level: "warn" }],
  });
}
