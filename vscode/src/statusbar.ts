// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log, TargetProfile } from "qsharp-lang";
import * as vscode from "vscode";
import { isQsharpDocument, qsharpExtensionId } from "./common";
import { getTarget, getTargetFriendlyName, setTarget } from "./config";
import { getActiveQSharpDocumentUri } from "./programConfig";

export function activateTargetProfileStatusBarItem(): vscode.Disposable[] {
  const disposables = [];

  disposables.push(registerTargetProfileCommand());

  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200,
  );
  disposables.push(statusBarItem);

  statusBarItem.command = `${qsharpExtensionId}.setTargetProfile`;

  disposables.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor && isQsharpDocument(editor.document)) {
        refreshStatusBarItemValue();
      } else if (editor?.document.uri.scheme !== "output") {
        // The output window counts as a text editor.
        // Avoid hiding the status bar if the focus is
        // on the output window.
        // https://github.com/Microsoft/vscode/issues/58869

        // Hide the status bar if we switched away from a
        // Q# document.
        statusBarItem.hide();
      }
    }),
  );

  disposables.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        getActiveQSharpDocumentUri() &&
        event.affectsConfiguration("Q#.qir.targetProfile")
      ) {
        refreshStatusBarItemValue();
      }
    }),
  );

  if (getActiveQSharpDocumentUri()) {
    refreshStatusBarItemValue();
  }

  function refreshStatusBarItemValue() {
    // The target profile setting is a "window" scoped setting,
    // meaning it can't be set on a per-folder basis. So we don't
    // need to pass a specific scope here to retrieve the document
    // value - we just use the workspace setting.
    // VS Code will return the default value defined by the extension
    // if none was set by the user, so targetProfile should always
    // be a valid string.
    const targetProfile = getTarget();

    statusBarItem.text = getTargetFriendlyName(targetProfile);
    statusBarItem.tooltip = new vscode.MarkdownString(`## Q# target profile
  The target profile determines the set of operations that are available to Q#
  programs, in order to generate valid QIR for the target platform. For more
  details see <https://aka.ms/qdk.qir>.`);
    statusBarItem.show();
  }

  return disposables;
}

function registerTargetProfileCommand() {
  return vscode.commands.registerCommand(
    `${qsharpExtensionId}.setTargetProfile`,
    async () => {
      const target = await vscode.window.showQuickPick(
        targetProfiles.map((profile) => ({
          label: profile.uiText,
        })),
        { placeHolder: "Select the QIR target profile" },
      );

      if (target) {
        setTarget(getTargetProfileSetting(target.label));
      }
    },
  );
}

const targetProfiles = [
  { configName: "base", uiText: "Q#: QIR base" },
  { configName: "adaptive_ri", uiText: "Q#: QIR Adaptive RI" },
  { configName: "unrestricted", uiText: "Q#: unrestricted" },
];

function getTargetProfileSetting(uiText: string): TargetProfile {
  switch (uiText) {
    case "Q#: QIR base":
      return "base";
    case "Q#: QIR Adaptive RI":
      return "adaptive_ri";
    case "Q#: unrestricted":
      return "unrestricted";
    default:
      log.error("invalid target profile found");
      return "unrestricted";
  }
}
