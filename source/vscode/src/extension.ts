// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  getLibrarySourceContent,
  log,
  qsharpGithubUriScheme,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import * as vscode from "vscode";
import { initAzureWorkspaces } from "./azure/commands.js";
import { CircuitEditorProvider } from "./circuitEditor.js";
import { initProjectCreator } from "./createProject.js";
import { activateDebugger } from "./debugger/activate.js";
import { startOtherQSharpDiagnostics } from "./diagnostics.js";
import { registerGhCopilotInstructionsCommand } from "./gh-copilot/instructions.js";
import { registerLanguageModelTools } from "./gh-copilot/tools.js";
import { activateLanguageService } from "./language-service/activate.js";
import {
  Logging,
  initLogForwarder,
  initOutputWindowLogger,
} from "./logging.js";
import { initFileSystem } from "./memfs.js";
import {
  registerCreateNotebookCommand,
  registerQSharpNotebookHandlers,
} from "./notebook.js";
import { getGithubSourceContent, setGithubEndpoint } from "./projectSystem.js";
import { initCodegen } from "./qirGeneration.js";
import { activateTargetProfileStatusBarItem } from "./statusbar.js";
import { initTelemetry } from "./telemetry.js";
import { registerWebViewCommands } from "./webviewPanel.js";

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

  context.subscriptions.push(...(await activateLanguageService(context)));

  context.subscriptions.push(...startOtherQSharpDiagnostics());

  context.subscriptions.push(...registerQSharpNotebookHandlers());

  context.subscriptions.push(CircuitEditorProvider.register(context));

  await initAzureWorkspaces(context);
  initCodegen(context);
  await activateDebugger(context);
  registerCreateNotebookCommand(context);
  registerWebViewCommands(context);
  await initFileSystem(context);
  await initProjectCreator(context);
  registerLanguageModelTools(context);
  // fire-and-forget
  registerGhCopilotInstructionsCommand(context);

  // The latest version for which we want to show the What's New page
  const WHATSNEW_VERSION = "defined5"; // <-- Update this when you want to show a new What's New

  const lastWhatsNewVersion = context.globalState.get<string>(
    "qdk.lastWhatsNewVersion",
  );
  const suppressUpdateNotifications = vscode.workspace
    .getConfiguration("Q#")
    .get<boolean>("notifications.suppressUpdateNotifications");

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.showWhatsNew", async () => {
      const whatsNewUri = vscode.Uri.joinPath(
        context.extensionUri,
        "WHATSNEW.md",
      );
      let markdown = "";
      try {
        const bytes = await vscode.workspace.fs.readFile(whatsNewUri);
        markdown = new TextDecoder("utf-8").decode(bytes);
      } catch (err) {
        log.error(`Failed to read WHATSNEW.md: ${err}`);
        markdown = "# What's New\nUnable to load release notes.";
      }
      const panel = vscode.window.createWebviewPanel(
        "qsharpWhatsNew",
        "What's New in QDK",
        vscode.ViewColumn.One,
        { enableScripts: true },
      );
      // Use client-side marked.js to render markdown
      panel.webview.html = `
        <!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>What's New in QDK</title>
          <style>
            body { font-family: var(--vscode-font-family); padding: 2em; color: var(--vscode-editor-foreground); background: var(--vscode-editor-background); }
            h1, h2, h3, h4, h5, h6 { color: var(--vscode-editor-foreground); }
            a { color: var(--vscode-textLink-foreground); }
          </style>
          <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
        </head>
        <body>
          <div id="content"></div>
          <script>
            const markdown = ${JSON.stringify(markdown)};
            document.getElementById('content').innerHTML = window.marked.parse(markdown);
          </script>
        </body>
        </html>
      `;
    }),
  );

  // This is just for debugging purposes, to ensure the What's New page is shown on
  // local execution of the extension.
  // const currentVersion = vscode.extensions.getExtension(
  //   "quantum.qsharp-lang-vscode-dev",
  // )?.packageJSON.version;

  // Show prompt after update if not suppressed
  if (
    lastWhatsNewVersion !== WHATSNEW_VERSION && // || currentVersion === "0.0.0") &&
    !suppressUpdateNotifications
  ) {
    await context.globalState.update(
      "qdk.lastWhatsNewVersion",
      WHATSNEW_VERSION,
    );
    // Only show prompt if not first install (i.e., lastWhatsNewVersion is not undefined/null)
    if (lastWhatsNewVersion !== undefined) {
      const buttons = ["What's New?", "Don't show this again"];
      const choice = await vscode.window.showInformationMessage(
        "The Azure Quantum Development Kit has been updated.",
        ...buttons,
      );
      if (choice === buttons[0]) {
        await vscode.commands.executeCommand("qsharp-vscode.showWhatsNew");
      } else if (choice === buttons[1]) {
        await vscode.workspace
          .getConfiguration("Q#")
          .update(
            "notifications.suppressUpdateNotifications",
            true,
            vscode.ConfigurationTarget.Global,
          );
        vscode.window.showInformationMessage(
          "You will no longer receive What's New notifications. You can re-enable them from the Q# settings.",
        );
      }
    } else {
      // First install or no previous version, just show What's New
      await vscode.commands.executeCommand("qsharp-vscode.showWhatsNew");
    }
  }

  log.info("Q# extension activated.");

  return api;
}

export interface ExtensionApi {
  // Only available in test mode. Allows listening to extension log events.
  logging?: Logging;
  setGithubEndpoint: (endpoint: string) => void;
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
