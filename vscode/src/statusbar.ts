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
    200
  );
  disposables.push(statusBarItem);

  statusBarItem.command = "quantum-set-target";

  disposables.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor && isQsharpDocument(editor.document)) {
        refreshStatusBarItemValue();
      } else {
        // Could still be showing if the document language was changed away from Q#
        statusBarItem.hide();
      }
    })
  );

  disposables.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        vscode.window.activeTextEditor &&
        isQsharpDocument(vscode.window.activeTextEditor.document) &&
        event.affectsConfiguration("qsharp.targetProfile")
      ) {
        refreshStatusBarItemValue();
      }
    })
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
      .getConfiguration("qsharp")
      .get<string>("targetProfile");

    statusBarItem.text = getTargetProfileUiText(targetProfile);
    statusBarItem.show();
  }

  return disposables;
}

function registerTargetProfileCommand() {
  return vscode.commands.registerCommand("quantum-set-target", async () => {
    const target = await vscode.window.showQuickPick(
      targetProfiles.map((profile) => profile.uiText),
      { placeHolder: "Select the QIR target profile" }
    );

    if (target) {
      vscode.workspace
        .getConfiguration("qsharp")
        .update(
          "targetProfile",
          getTargetProfileSetting(target),
          vscode.ConfigurationTarget.Global
        );
    }
  });
}

const targetProfiles = [
  { configName: "base", uiText: "QIR:Base" },
  { configName: "full", uiText: "QIR:Full" },
];

function getTargetProfileUiText(targetProfile?: string) {
  switch (targetProfile) {
    case "base":
      return "QIR:Base";
    case "full":
      return "QIR:Full";
    default:
      log.error("invalid target profile found");
      return "QIR:Invalid";
  }
}

function getTargetProfileSetting(uiText: string) {
  switch (uiText) {
    case "QIR:Base":
      return "base";
    case "QIR:Full":
      return "full";
    default:
      log.error("invalid target profile found");
      return "full";
  }
}
