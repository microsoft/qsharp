// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  getLanguageService,
  getLibrarySourceContent,
  loadWasmModule,
  log,
  qsharpGithubUriScheme,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import * as vscode from "vscode";
import { initAzureWorkspaces } from "./azure/commands.js";
import { createCodeActionsProvider } from "./codeActions.js";
import { createCodeLensProvider } from "./codeLens.js";
import {
  isQsharpDocument,
  isQsharpNotebookCell,
  qsharpLanguageId,
} from "./common.js";
import { createCompletionItemProvider } from "./completion";
import { getTarget } from "./config";
import { initProjectCreator } from "./createProject.js";
import { activateDebugger } from "./debugger/activate";
import { createDefinitionProvider } from "./definition";
import { startCheckingQSharp } from "./diagnostics";
import { createFormattingProvider } from "./format.js";
import { createHoverProvider } from "./hover";
import {
  Logging,
  initLogForwarder,
  initOutputWindowLogger,
} from "./logging.js";
import { initFileSystem } from "./memfs.js";
import {
  registerCreateNotebookCommand,
  registerQSharpNotebookCellUpdateHandlers,
  registerQSharpNotebookHandlers,
} from "./notebook.js";
import {
  fetchGithubRaw,
  findManifestDirectory,
  getGithubSourceContent,
  listDirectory,
  readFile,
  resolvePath,
  setGithubEndpoint,
} from "./projectSystem.js";
import { initCodegen } from "./qirGeneration.js";
import { createReferenceProvider } from "./references.js";
import { createRenameProvider } from "./rename.js";
import { createSignatureHelpProvider } from "./signature.js";
import { activateTargetProfileStatusBarItem } from "./statusbar.js";
import {
  EventType,
  QsharpDocumentType,
  initTelemetry,
  sendTelemetryEvent,
} from "./telemetry.js";
import { registerWebViewCommands } from "./webviewPanel.js";
import { activateChatParticipant } from "./copilot/chatParticipant.js";

export async function activate(
  context: vscode.ExtensionContext,
): Promise<ExtensionApi> {
  const api: ExtensionApi = { setGithubEndpoint };

  if (context.extensionMode === vscode.ExtensionMode.Test) {
    // Don't log to the output window in tests, forward to a listener instead
    api.logging = initLogForwarder();
  } else {
    // Direct logging to the output window
    initOutputWindowLogger();
  }

  log.info("Q# extension activating.");
  initTelemetry(context);

  checkForOldQdk();

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.qdkCopilot", () => {
      // Create and show a new webview
      let panel = vscode.window.createWebviewPanel(
        "qdkCopilot", // Identifies the type of the webview. Used internally
        "QDK Copilot", // Title of the panel displayed to the user
        vscode.ViewColumn.One, // Editor column to show the new webview panel in.
        {}, // Webview options. More on these later.
      );

      panel.webview.html = `
<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
  <semantics>
    <mrow>
      <mfrac>
        <mi>i</mi>
        <msqrt>
          <mn>2</mn>
        </msqrt>
      </mfrac>
      <mo>⋅</mo>
      <mrow>
        <mo fence="true">[</mo>
        <mtable rowspacing="0.16em" columnalign="center center" columnspacing="1em">
          <mtr>
            <mtd>
              <mstyle scriptlevel="0" displaystyle="false">
                <mn>1</mn>
              </mstyle>
            </mtd>
            <mtd>
              <mstyle scriptlevel="0" displaystyle="false">
                <mn>0</mn>
              </mstyle>
            </mtd>
          </mtr>
          <mtr>
            <mtd>
              <mstyle scriptlevel="0" displaystyle="false">
                <mn>0</mn>
              </mstyle>
            </mtd>
            <mtd>
              <mstyle scriptlevel="0" displaystyle="false">
                <mn>1</mn>
              </mstyle>
            </mtd>
          </mtr>
        </mtable>
        <mo fence="true">]</mo>
      </mrow>
    </mrow>
    <annotation encoding="application/x-tex">{i \over \sqrt{2}} \cdot \begin{bmatrix}1 &amp; 0 \\ 0 &amp; 1\end{bmatrix}</annotation>
  </semantics>
</math>`;
    }),
  );

  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(
      qsharpLibraryUriScheme,
      new QsTextDocumentContentProvider(),
    ),
  );

  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(
      qsharpGithubUriScheme,
      {
        provideTextDocumentContent(uri) {
          return getGithubSourceContent(uri);
        },
      },
    ),
  );

  context.subscriptions.push(...activateTargetProfileStatusBarItem());

  context.subscriptions.push(
    ...(await activateLanguageService(context.extensionUri)),
  );

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  initAzureWorkspaces(context);
  initCodegen(context);
  activateDebugger(context);
  registerCreateNotebookCommand(context);
  registerWebViewCommands(context);
  initFileSystem(context);
  initProjectCreator(context);
  activateChatParticipant(context);

  log.info("Q# extension activated.");

  return api;
}

export interface ExtensionApi {
  // Only available in test mode. Allows listening to extension log events.
  logging?: Logging;
  setGithubEndpoint: (endpoint: string) => void;
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

async function activateLanguageService(extensionUri: vscode.Uri) {
  const subscriptions: vscode.Disposable[] = [];

  const languageService = await loadLanguageService(extensionUri);

  // diagnostics
  subscriptions.push(...startCheckingQSharp(languageService));

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
      "@", // for attribute completion
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

function registerConfigurationChangeHandlers(
  languageService: ILanguageService,
) {
  return vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration("Q#.qir.targetProfile")) {
      updateLanguageServiceProfile(languageService);
    }
  });
}

export class QsTextDocumentContentProvider
  implements vscode.TextDocumentContentProvider
{
  onDidChange?: vscode.Event<vscode.Uri> | undefined;

  provideTextDocumentContent(uri: vscode.Uri): vscode.ProviderResult<string> {
    return getLibrarySourceContent(uri.toString());
  }
}

function checkForOldQdk() {
  const oldQdkExtension = vscode.extensions.getExtension(
    "quantum.quantum-devkit-vscode",
  );

  const prereleaseQdkExtension = vscode.extensions.getExtension(
    "quantum.qsharp-lang-vscode-dev",
  );

  const releaseQdkExtension = vscode.extensions.getExtension(
    "quantum.qsharp-lang-vscode",
  );

  const previousQdkWarningMessage =
    'Extension "Microsoft Quantum Development Kit for Visual Studio" (`quantum.quantum-devkit-vscode`) found. We recommend uninstalling the prior QDK before using this release.';

  const bothReleaseAndPrereleaseWarningMessage =
    'Extension "Azure Quantum Development Kit (QDK)" has both release and pre-release versions installed. We recommend uninstalling one of these versions.';

  // we don't await the warnings below so we don't block extension initialization
  if (oldQdkExtension) {
    log.warn(previousQdkWarningMessage);
    vscode.window.showWarningMessage(previousQdkWarningMessage);
  }

  if (prereleaseQdkExtension && releaseQdkExtension) {
    log.warn(bothReleaseAndPrereleaseWarningMessage);
    vscode.window.showWarningMessage(bothReleaseAndPrereleaseWarningMessage);
  }
}
