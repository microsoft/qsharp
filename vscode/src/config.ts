// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

export function getTarget(): string {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<string>("targetProfile", "full");
  return target;
}

export async function setTarget(target: string) {
  const config = vscode.workspace.getConfiguration("Q#");
  await config.update(
    "targetProfile",
    target,
    vscode.ConfigurationTarget.Global,
  );
}
