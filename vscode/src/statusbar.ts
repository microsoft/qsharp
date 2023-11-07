// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { isQsharpDocument } from "./common";

export function activateTargetProfileStatusBarItem(): vscode.Disposable[] {
  const disposables = [];

  disposables.push(registerTargetProfileCommand());

  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200,
  );
  disposables.push(statusBarItem);

  statusBarItem.command = "qsharp-vscode.setTargetProfile";

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
        vscode.window.activeTextEditor &&
        isQsharpDocument(vscode.window.activeTextEditor.document) &&
        event.affectsConfiguration("Q#.targetProfile")
      ) {
        refreshStatusBarItemValue();
      }
    }),
  );

  if (
    vscode.window.activeTextEditor &&
    isQsharpDocument(vscode.window.activeTextEditor.document)
  ) {
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
    const targetProfile = vscode.workspace
      .getConfiguration("Q#")
      .get<string>("targetProfile", "unrestricted");

    statusBarItem.text = getTargetProfileUiText(targetProfile);
    statusBarItem.show();
  }

  return disposables;
}

function initializeTargetProfile() {
  const configuration = vscode.workspace.getConfiguration("Q#");
  const current = configuration.get<string>("targetProfile", "unset");
  // If the target profile is unset or full, set it to unrestricted.
  if (current == "unset" || current == "full") {
    configuration.update(
      "targetProfile",
      "unrestricted",
      vscode.ConfigurationTarget.Global,
    );
  }
}

function registerTargetProfileCommand() {
  initializeTargetProfile();

  return vscode.commands.registerCommand(
    "qsharp-vscode.setTargetProfile",
    async () => {
      const target = await vscode.window.showQuickPick(
        targetProfiles.map((profile) => profile.uiText),
        { placeHolder: "Select the QIR target profile" },
      );

      if (target) {
        vscode.workspace
          .getConfiguration("Q#")
          .update(
            "targetProfile",
            getTargetProfileSetting(target),
            vscode.ConfigurationTarget.Global,
          );
      }
    },
  );
}

const targetProfiles = [
  { configName: "base", uiText: "QIR:Base" },
  { configName: "unrestricted", uiText: "QIR:Unrestricted" },
];

function getTargetProfileUiText(targetProfile?: string) {
  switch (targetProfile) {
    case "base":
      return "QIR:Base";
    case "unrestricted":
      return "QIR:Unrestricted";
    default:
      log.error("invalid target profile found");
      return "QIR:Invalid";
  }
}

function getTargetProfileSetting(uiText: string) {
  switch (uiText) {
    case "QIR:Base":
      return "base";
    case "QIR:Unrestricted":
      return "unrestricted";
    default:
      log.error("invalid target profile found");
      return "unrestricted";
  }
}
