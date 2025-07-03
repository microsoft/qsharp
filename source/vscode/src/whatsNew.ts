// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

// The latest version for which we want to show the What's New page
const WHATSNEW_VERSION = undefined; // <-- Update this when you want to show a new What's New

export function registerWhatsNewCommand(
  context: vscode.ExtensionContext,
): vscode.Disposable[] {
  return [
    vscode.commands.registerCommand("qsharp-vscode.showWhatsNew", async () => {
      const whatsNewUri = vscode.Uri.joinPath(
        context.extensionUri,
        "WHATSNEW.md",
      );
      await vscode.commands.executeCommand(
        "markdown.showPreview",
        whatsNewUri,
        vscode.ViewColumn.One,
        { locked: true },
      );
    }),
  ];
}

export async function maybeShowWhatsNewPrompt(
  context: vscode.ExtensionContext,
) {
  const lastWhatsNewVersion = context.globalState.get<string>(
    "qdk.lastWhatsNewVersion",
  );
  const suppressUpdateNotifications = vscode.workspace
    .getConfiguration("Q#")
    .get<boolean>("notifications.suppressUpdateNotifications");

  if (
    lastWhatsNewVersion !== WHATSNEW_VERSION &&
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
}
