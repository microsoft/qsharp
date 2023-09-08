// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
import { isQsharpDocument } from "./common";

export function createQirStatusBarItem(): vscode.Disposable[] {
  const disposables = [];
  const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    200
  );
  disposables.push(statusBarItem);

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
    const targetProfile = vscode.workspace
      .getConfiguration("qsharp")
      .get<string>("targetProfile");
    switch (targetProfile) {
      case "base":
        statusBarItem.text = "QIR:Base";
        break;
      case "full":
        statusBarItem.text = "QIR:Full";
        break;
      default:
        log.error("invalid target profile found in settings");
        statusBarItem.text = "QIR:Invalid";
        break;
    }
    statusBarItem.show();
  }

  return disposables;
}
