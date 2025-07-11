// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

// The latest version for which we want to show the changelog page
const CHANGELOG_VERSION = undefined; // <-- Update this when you want to show a new changelog to users

export function registerChangelogCommand(
  context: vscode.ExtensionContext,
): vscode.Disposable[] {
  return [
    vscode.commands.registerCommand("qsharp-vscode.showChangelog", async () => {
      const changelogUri = vscode.Uri.joinPath(
        context.extensionUri,
        "CHANGELOG.md",
      );
      await vscode.commands.executeCommand(
        "markdown.showPreview",
        changelogUri,
        vscode.ViewColumn.One,
        { locked: true },
      );
    }),
  ];
}

export async function maybeShowChangelogPrompt(
  context: vscode.ExtensionContext,
) {
  const lastChangelogVersion = context.globalState.get<string>(
    "qdk.lastChangelogVersion",
  );
  const suppressUpdateNotifications = vscode.workspace
    .getConfiguration("Q#")
    .get<boolean>("notifications.suppressUpdateNotifications");

  if (
    lastChangelogVersion !== CHANGELOG_VERSION &&
    !suppressUpdateNotifications
  ) {
    await context.globalState.update(
      "qdk.lastChangelogVersion",
      CHANGELOG_VERSION,
    );
    // Only show prompt if not first install (i.e., lastChangelogVersion is not undefined/null)
    if (lastChangelogVersion !== undefined) {
      const buttons = ["What's New?", "Don't show this again"];
      const choice = await vscode.window.showInformationMessage(
        "The Azure Quantum Development Kit has been updated.",
        ...buttons,
      );
      if (choice === buttons[0]) {
        await vscode.commands.executeCommand("qsharp-vscode.showChangelog");
      } else if (choice === buttons[1]) {
        await vscode.workspace
          .getConfiguration("Q#")
          .update(
            "notifications.suppressUpdateNotifications",
            true,
            vscode.ConfigurationTarget.Global,
          );
        vscode.window.showInformationMessage(
          'You will no longer receive "What\'s New" notifications. You can re-enable them from the Q# settings.',
        );
      }
    } else {
      // First install or no previous version, just show changelog
      await vscode.commands.executeCommand("qsharp-vscode.showChangelog");
    }
  }
}
